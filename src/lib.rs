use jupiter_amm_interface::{
    AccountProvider, Amm, AmmContext, AmmError, AmmLabel, KeyedAccount, Quote, QuoteParams,
    SingleProgramAmm, Swap, SwapAndAccountMetas, SwapParams, single_program_amm,
};
use rust_decimal::Decimal;
use solana_account::ReadableAccount;
use solana_instruction::AccountMeta;
use solana_pubkey::Pubkey;

pub const PROGRAM_ID: Pubkey =
    solana_pubkey::pubkey!("H6UKMGJB2ge93xvcE2WyoLDWYKtyqAKXKGid4nath757");
const POOL_ACCOUNT_LEN: usize = 90;
const POOL_DISC: [u8; 8] = [241, 154, 109, 4, 17, 177, 109, 188];
const SWAP_A_FOR_B_DISC: [u8; 8] = [136, 212, 69, 214, 198, 184, 9, 232];
const SWAP_B_FOR_A_DISC: [u8; 8] = [153, 32, 185, 3, 110, 47, 40, 194];
const TOKEN_PROGRAM_ID: Pubkey =
    solana_pubkey::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

#[derive(Clone, Copy)]
pub struct AarushDexAmm {
    key: Pubkey,
    token_a: Pubkey,
    token_b: Pubkey,
    fee_bps: u16,
    reserve_a: u64,
    reserve_b: u64,
}

single_program_amm!(AarushDexAmm, PROGRAM_ID, "AarushDEX");

fn read_u16(d: &[u8], off: usize) -> u16 {
    u16::from_le_bytes(d[off..off + 2].try_into().unwrap())
}

fn read_u64(d: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(d[off..off + 8].try_into().unwrap())
}

fn read_pubkey(d: &[u8], off: usize) -> Pubkey {
    Pubkey::try_from(&d[off..off + 32]).unwrap()
}

fn parse_pool(key: Pubkey, data: &[u8]) -> Result<AarushDexAmm, AmmError> {
    if data.len() != POOL_ACCOUNT_LEN {
        return Err(AmmError::from(
            "AarushDEX Pool account length is not 90 bytes",
        ));
    }
    if data[0..8] != POOL_DISC {
        return Err(AmmError::from("account is not an AarushDEX Pool"));
    }
    Ok(AarushDexAmm {
        key,
        token_a: read_pubkey(data, 8),
        token_b: read_pubkey(data, 40),
        fee_bps: read_u16(data, 72),
        reserve_a: read_u64(data, 74),
        reserve_b: read_u64(data, 82),
    })
}

fn vault_a(pool: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"vault_a", pool.as_ref()], &PROGRAM_ID).0
}

fn vault_b(pool: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"vault_b", pool.as_ref()], &PROGRAM_ID).0
}

fn cp_out(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_bps: u16,
) -> Result<(u64, u64), AmmError> {
    if amount_in == 0 || reserve_in == 0 || reserve_out == 0 {
        return Ok((0, 0));
    }

    if fee_bps > 10_000 {
        return Err(AmmError::from("invalid fee_bps"));
    }

    let amount = amount_in as u128;
    let fee_factor = 10_000u128 - fee_bps as u128;

    let amount_less_fee = amount
        .checked_mul(fee_factor)
        .ok_or_else(|| AmmError::from("swap math overflow"))?;

    let numerator = amount_less_fee
        .checked_mul(reserve_out as u128)
        .ok_or_else(|| AmmError::from("swap math overflow"))?;

    let denominator = (reserve_in as u128)
        .checked_mul(10_000)
        .and_then(|x| x.checked_add(amount_less_fee))
        .ok_or_else(|| AmmError::from("swap math overflow"))?;

    if denominator == 0 {
        return Err(AmmError::from("swap math denominator is zero"));
    }

    let out = numerator / denominator;

    let fee = amount
        .checked_mul(fee_bps as u128)
        .and_then(|x| x.checked_div(10_000))
        .ok_or_else(|| AmmError::from("fee math overflow"))?;

    Ok((
        out.min(u64::MAX as u128) as u64,
        fee.min(u64::MAX as u128) as u64,
    ))
}

impl Amm for AarushDexAmm {
    fn from_keyed_account(
        keyed_account: &KeyedAccount,
        _amm_context: &AmmContext,
    ) -> Result<Self, AmmError> {
        if keyed_account.account.owner() != &PROGRAM_ID {
            return Err(AmmError::from("pool is not owned by AarushDEX"));
        }
        parse_pool(keyed_account.key, keyed_account.account.data())
    }

    fn label(&self) -> AmmLabel {
        Self::LABEL
    }

    fn program_id(&self) -> Pubkey {
        PROGRAM_ID
    }

    fn key(&self) -> Pubkey {
        self.key
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        vec![self.token_a, self.token_b]
    }

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        vec![self.key]
    }

    fn update(&mut self, account_provider: impl AccountProvider) -> Result<(), AmmError> {
        let refreshed = {
            let account = account_provider.try_get(&self.key)?;
            parse_pool(self.key, account.data())?
        };
        *self = refreshed;
        Ok(())
    }

    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote, AmmError> {
        if quote_params.swap_mode != jupiter_amm_interface::SwapMode::ExactIn {
            return Err(AmmError::from(
                "AarushDEX adapter currently supports ExactIn only",
            ));
        }

        let (reserve_in, reserve_out, fee_mint) = if quote_params.input_mint == self.token_a
            && quote_params.output_mint == self.token_b
        {
            (self.reserve_a, self.reserve_b, self.token_a)
        } else if quote_params.input_mint == self.token_b
            && quote_params.output_mint == self.token_a
        {
            (self.reserve_b, self.reserve_a, self.token_b)
        } else {
            return Err(AmmError::from("input/output mints do not match this pool"));
        };

        let (out, fee) = cp_out(quote_params.amount, reserve_in, reserve_out, self.fee_bps)?;
        Ok(Quote {
            in_amount: quote_params.amount,
            out_amount: out,
            fee_amount: fee,
            fee_mint,
            fee_pct: Decimal::from(self.fee_bps) / Decimal::from(10_000u64),
        })
    }

    fn get_swap_and_account_metas(
        &self,
        swap_params: &SwapParams,
    ) -> Result<SwapAndAccountMetas, AmmError> {
        let a_to_b =
            swap_params.source_mint == self.token_a && swap_params.destination_mint == self.token_b;
        let b_to_a =
            swap_params.source_mint == self.token_b && swap_params.destination_mint == self.token_a;

        if !a_to_b && !b_to_a {
            return Err(AmmError::from("swap mints do not match this pool"));
        }

        // These are the exact 9 accounts verified by the deployed IDL before this file is created:
        // user, pool, vault_a, vault_b, user_a, user_b, token_program.
        // The user is both the Signer and the owner of the user's source token account.
        let account_metas = vec![
            // 0: user
            AccountMeta::new(swap_params.user, true),
            // 1: pool
            AccountMeta::new(self.key, false),
            // 2: vault_a
            AccountMeta::new(vault_a(&self.key), false),
            // 3: vault_b
            AccountMeta::new(vault_b(&self.key), false),
            // 4: user_a
            AccountMeta::new(
                if a_to_b {
                    swap_params.source_token_account
                } else {
                    swap_params.destination_token_account
                },
                false,
            ),
            // 5: user_b
            AccountMeta::new(
                if a_to_b {
                    swap_params.destination_token_account
                } else {
                    swap_params.source_token_account
                },
                false,
            ),
            // 6: token_a_mint
            AccountMeta::new_readonly(self.token_a, false),
            // 7: token_b_mint
            AccountMeta::new_readonly(self.token_b, false),
            // 8: token_program
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        ];

        Ok(SwapAndAccountMetas {
            swap: Swap::Placeholder {
                data: Some(vec![if a_to_b { 1 } else { 0 }]),
            },
            account_metas,
        })
    }

    fn supports_exact_out(&self) -> bool {
        false
    }

    fn get_accounts_len(&self) -> usize {
        9
    }

    fn program_dependencies(&self) -> Vec<(Pubkey, String)> {
        vec![(TOKEN_PROGRAM_ID, "spl_token".to_string())]
    }
}

pub fn encode_native_swap(swap: &Swap, in_amount: u64) -> Vec<u8> {
    let direction = match swap {
        Swap::Placeholder { data: Some(d) } if d.as_slice() == [1] => true,
        Swap::Placeholder { data: Some(d) } if d.as_slice() == [0] => false,
        _ => panic!("unexpected AarushDEX swap placeholder"),
    };

    let disc = if direction {
        SWAP_A_FOR_B_DISC
    } else {
        SWAP_B_FOR_A_DISC
    };
    let mut data = Vec::with_capacity(24);
    data.extend_from_slice(&disc);
    data.extend_from_slice(&in_amount.to_le_bytes());
    data.extend_from_slice(&0u64.to_le_bytes());
    data
}
