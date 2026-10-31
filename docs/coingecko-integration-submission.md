# AarushDEX — CoinGecko Spot Exchange API Submission

> **Status: DRAFT — NOT READY FOR COINGECKO SUBMISSION.**  
> This is a public implementation specification, not a statement that the REST endpoints below are live. No public CoinGecko-compatible REST API was present in the AarushDEX repository inspected on 31 October 2026. Replace every “Not implemented” item with a tested HTTPS URL after deployment.

## Exchange classification

- **Exchange type:** Decentralized Spot Exchange
- **DEX model:** Constant-product AMM
- **Network:** Solana mainnet
- **AarushDEX program ID:** `H6UKMGJB2ge93xvcE2WyoLDWYKtyqAKXKGid4nath757`
- **Example pool account documented in the public SDK repository:** `7tojge8bQy7HdQmXLwEFZU33Jd3Wch3pNEdXrLK3aLPw`
- **Public implementation reference:** [AarushDEX Jupiter AMM SDK](https://github.com/aarushasoffical/aarushdex-jupiter)

The repository's IDL describes pool creation, add/remove liquidity, and both swap directions. The SDK describes exact-in swaps and constant-product pricing. It does not provide the CoinGecko REST service, market-wide pool indexing, 24-hour trade aggregation, or USD valuation service.

## CoinGecko submission template — Spot Exchanges

| **Public API Documentation** | **[Mandatory]** | **Not implemented.** This file documents the proposed API only; it is not live API documentation. |
|---|---|---|
| **Tickers** | **[Mandatory]** | Proposed route: `GET https://<API_HOST>/api/v1/coingecko/tickers` — **not deployed** |
| **Pairs** | [Optional] | Not provided. A separate pairs endpoint is not required by the supplied CoinGecko API standard. |
| **Orderbook** | [Optional for AMM DEX] | Not provided. AarushDEX is an AMM. The proposed ±2% depth formula is documented below and must be tested against the deployed program before submission. |
| **Historical Trade Data** | [Optional for AMM DEX] | Not provided. Historical trade indexing and the endpoint are not implemented. |

### Derivative Exchanges

| **Public API Documentation** | **[Mandatory]** | Not applicable — AarushDEX is a spot-only AMM. |
|---|---|---|
| **Contracts** | [Mandatory] | N/A — no derivative contracts |
| **Contract Specs** | [Mandatory] | N/A — no derivative contracts |
| **Orderbook** | [Mandatory] | N/A — no derivative orderbook |

## Proposed mandatory spot endpoint

### `GET /api/v1/coingecko/tickers`

**Authentication:** none.  
**Format:** JSON.  
**Availability:** public HTTPS; rate limits must permit at least minutely polling.  
**Response:** a JSON array containing one row for each supported, active pool. The endpoint must enumerate all qualifying pools, not just the example pool.

Example shape only — the values below are placeholders, not live market data:

```json
[
  {
    "ticker_id": "<BASE_MINT>_<TARGET_MINT>",
    "base_currency": "<BASE_MINT>",
    "target_currency": "<TARGET_MINT>",
    "last_price": "<TARGET_UNITS_PER_BASE_UNIT>",
    "base_volume": "<24H_BASE_VOLUME>",
    "target_volume": "<24H_TARGET_VOLUME>",
    "pool_id": "<POOL_ACCOUNT_PUBLIC_KEY>",
    "liquidity_in_usd": "<TOTAL_POOL_LIQUIDITY_USD>"
  }
]
```

### Field mapping

| Field | CoinGecko status | AarushDEX implementation rule |
|---|---|---|
| `ticker_id` | Mandatory | Stable market identifier using base and target mint addresses separated by `_`. If multiple pools can exist for the same mint pair, append a documented pool identifier to keep each ticker unique. |
| `base_currency` | Mandatory | Base token's Solana mint address. |
| `target_currency` | Mandatory | Target token's Solana mint address. |
| `last_price` | Mandatory | Most recent completed on-chain trade price, expressed as target-token units per one base-token unit. |
| `base_volume` | Mandatory | Rolling 24-hour single-sided traded volume expressed in base-token units. |
| `target_volume` | Mandatory | Rolling 24-hour single-sided traded volume expressed in target-token units. |
| `pool_id` | Mandatory for DEX | Solana public key of the pool account. |
| `liquidity_in_usd` | Mandatory for DEX | Total pool value: base reserve × base USD price + target reserve × target USD price. Do not report one-sided liquidity as total liquidity. |
| `bid` | Recommended | Omit unless a reliable, website-consistent AMM quote definition is implemented. |
| `ask` | Recommended | Omit unless a reliable, website-consistent AMM quote definition is implemented. |
| `high` | Recommended | Highest completed trade price over the rolling 24-hour window. |
| `low` | Recommended | Lowest completed trade price over the rolling 24-hour window. |

All token amounts must be converted from raw integer units using each mint's actual decimals before returning decimal values. For DEX markets, use mint addresses for `base_currency` and `target_currency`, not ticker symbols alone.

The 24-hour volume calculation must count each completed swap once. For each swap, aggregate the amount traded on the base side into `base_volume` and the corresponding amount on the target side into `target_volume`; do not double-count a swap by adding both its input and output as separate trades.

## AMM depth formula in place of an orderbook endpoint

CoinGecko's supplied standard says an AMM DEX may omit `/orderbook` if it supplies `liquidity_in_usd` in `/tickers` and provides the formula used to derive ±2% depth.

AarushDEX uses constant-product pricing in its public Jupiter adapter. The following is a **proposed fee-aware constant-product calculation**, not yet verified against the deployed swap instruction or CoinGecko's depth calculation. It must be checked against actual on-chain swap results and the values shown in the trading interface before being submitted.

Let:

- (X) = current human-unit base-token reserve.
- (Y) = current human-unit target-token reserve.
- (P_0 = Y/X) = current marginal price in target units per base unit.
- (f = 	ext{fee\_bps}/10{,}000) = input fee fraction, if this is confirmed to match the deployed swap logic.
- (USD_B), (USD_T) = independently sourced USD prices for the base and target tokens.

For (R > 1), define:

[
T(R,f)=\frac{-(2-f)+\sqrt{(2-f)^2+4(1-f)(R-1)}}{2(1-f)}
]

Then the input-side depth estimates are:

- **Price rises by 2%:** target-token input required is (Y \times T(1.02,f)). USD depth is (Y \times T(1.02,f) \times USD_T).
- **Price falls by 2%:** base-token input required is (X \times T(1/0.98,f)). USD depth is (X \times T(1/0.98,f) \times USD_B).

This calculation assumes the input fee is applied exactly as shown and that the pool follows the constant-product swap equation. If the deployed program handles fees differently, the formula must be changed to match the program. USD prices must come from a documented, reliable source and must be consistent with the AarushDEX website. Do not publish estimated depth as verified data until parity tests pass.

## Data and operational requirements before submission

1. Implement a public HTTPS REST service and deploy it at a stable host. Replace `<API_HOST>` with the real host only after the route is live.
2. Enumerate all AarushDEX pool accounts owned by the program and verify each pool and its token mints/vault balances. The example pool above is not proof that it is the only pool.
3. Read mint decimals and convert all raw token amounts correctly.
4. Index successful on-chain swaps to calculate the last transacted price and rolling 24-hour base/target volumes. Use a unique trade identity if historical trades are later exposed; a timestamp alone is not a unique trade ID.
5. Provide a defensible USD valuation source for both sides of each pool and calculate total pool liquidity.
6. Keep the API unauthenticated, return JSON, and set rate limits that support polling at least once per minute.
7. Display the same last price, 24-hour volumes, and USD liquidity on the AarushDEX trading interface. CoinGecko's supplied standard says API values must match the website.
8. Add an information page explaining the data source and calculation methodology. Monitor endpoint uptime and data freshness.
9. Test every field and the depth formula against live on-chain state and the website before sharing the live API URLs with CoinGecko.

## DEX fork / factory / router section

AarushDEX should **not** claim to be one of the listed supported GeckoTerminal forks based on the information verified for this document. Its public repository identifies a custom Solana Anchor program; the current supported-fork list does not list AarushDEX. The program ID is included above for identification, but it is **not being represented here as a verified factory address or router address**.

| Field | Submission |
|---|---|
| **DEX fork type** | Custom Solana constant-product AMM; not identified as a supported fork |
| **Factory Address** | Not supplied — no separate, verified factory address established |
| **Router Address** | Not supplied — no separate, verified router address established |
| **GeckoTerminal URL (if any)** | Not listed / not verified |

For a non-EVM chain/DEX, CoinGecko's current GeckoTerminal guidance directs applicants to provide adapters following its API endpoint requirements. The exchange-listing requirements also say Spot DEX API documentation may not be mandatory when integration can be achieved through factory and router addresses. AarushDEX should confirm the correct route with CoinGecko for its custom Solana program; do not submit unverified factory/router values.

## References

- [AarushDEX Jupiter AMM SDK](https://github.com/aarushasoffical/aarushdex-jupiter)
- [CoinGecko Integration API Standards supplied for this submission](https://docs.google.com/document/d/1v27QFoQq1SKT3Priq3aqPgB70Xd_PnDzbOCiuoCyixw)
- [CoinGecko: How to Add a New Exchange](https://support.coingecko.com/articles/2005455880-how-to-add-a-new-exchange)
- [GeckoTerminal: DEX Forks supported by GeckoTerminal](https://support.coingecko.com/articles/7376867784-dex-forks-supported-by-geckoterminal)
- [GeckoTerminal: How to integrate Non-EVM Chain/DEX](https://support.coingecko.com/articles/1851457136-how-to-integrate-non-evm-chain-dex-on-geckoterminal)
