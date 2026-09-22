use aarushdex_jupiter::{AarushDexAmm, encode_native_swap};
use jupiter_amm_interface::Swap;
use jupiter_amm_test_kit::{PoolTest, assert_pool_parity};
use solana_pubkey::pubkey;

const TEST_POOL: solana_pubkey::Pubkey = pubkey!("7tojge8bQy7HdQmXLwEFZU33Jd3Wch3pNEdXrLK3aLPw");
const TOKEN_A: solana_pubkey::Pubkey = pubkey!("5eHSjLZahnEwSiWQizHp2ND69Wna3HjNJZEpQJMQhXLJ");
const TOKEN_B: solana_pubkey::Pubkey = pubkey!("5xFZDaJ2t1K8UKmngZFK1QUUZ22rL7pcc32i1GGRGouZ");

#[test]
fn aarushdex_live_pool_parity() {
    let test = PoolTest::new(TEST_POOL)
        .add_swap(TOKEN_A, TOKEN_B, 1000000)
        .add_swap(TOKEN_B, TOKEN_A, 9999);

    assert_pool_parity::<AarushDexAmm>(&test, encode_native_swap);
}

#[test]
fn aarushdex_encoder_has_both_directions() {
    let a = encode_native_swap(
        &Swap::Placeholder {
            data: Some(vec![1]),
        },
        123,
    );
    let b = encode_native_swap(
        &Swap::Placeholder {
            data: Some(vec![0]),
        },
        123,
    );
    assert_eq!(&a[..8], &[136, 212, 69, 214, 198, 184, 9, 232]);
    assert_eq!(&b[..8], &[153, 32, 185, 3, 110, 47, 40, 194]);
}
