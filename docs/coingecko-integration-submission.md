# AarushDEX — CoinGecko / GeckoTerminal Integration Submission

> **Status: DRAFT — not ready to submit as a live REST API.**  
> This document is a completed draft of the supplied Spot/Derivatives template, with implementation notes. It is not a live API reference. The public AarushDEX repository inspected on 31 October 2026 contains a Jupiter AMM adapter and program IDL, but no public CoinGecko-compatible REST service. Do not submit the proposed routes as working endpoints until they have been deployed and tested.

## Exchange classification

- **Exchange type:** Decentralized Spot Exchange
- **Market model:** Constant-product AMM
- **Network:** Solana mainnet
- **AarushDEX program ID:** `H6UKMGJB2ge93xvcE2WyoLDWYKtyqAKXKGid4nath757`
- **Example mainnet pool account:** `7tojge8bQy7HdQmXLwEFZU33Jd3Wch3pNEdXrLK3aLPw`
- **Public repository reference:** [AarushDEX Jupiter AMM SDK](https://github.com/aarushasoffical/aarushdex-jupiter)

The repository's IDL describes pool creation, add/remove liquidity, and both swap directions. The SDK implements exact-in quoting with constant-product pricing. The inspected repository does not contain a CoinGecko REST service, market-wide pool indexer, 24-hour trade aggregator, or USD valuation service.

## Spot Exchanges — completed submission template

| **Public API Documentation** | **[Mandatory in the supplied template]** | **Not available yet.** This document is an implementation draft, not public documentation for a deployed API. |
|---|---|---|
| **Tickers** | [Mandatory] | Proposed: `GET https://<API_HOST>/api/v1/coingecko/tickers` — **not implemented / no live URL** |
| **Pairs** | [Optional] | Not implemented. Optional under the supplied CoinGecko API standard. |
| **Orderbook** | [Optional for AMM DEX] | Not implemented. AarushDEX is an AMM; the proposed ±2% depth methodology is described below and must be validated against the deployed swap program. |
| **Historical Trade Data** | [Optional for AMM DEX] | Not implemented. No historical trade index or public historical-trades endpoint was found in the inspected repository. |

## Derivative Exchanges — completed submission template

| **Public API Documentation** | **[Mandatory]** | **Not applicable — AarushDEX is spot-only and has no derivative markets.** |
|---|---|---|
| **Contracts** | [Mandatory] | N/A — no futures, perpetuals, or options |
| **Contract Specs** | [Mandatory] | N/A — no derivative contracts |
| **Orderbook** | [Mandatory] | N/A — no derivative orderbook |

## Proposed Spot API specification

The following is the proposed REST contract if AarushDEX proceeds with a CoinGecko-style exchange API submission. These routes must not be represented as operational until implemented and publicly deployed.

### `GET /api/v1/coingecko/tickers`

- **Authentication:** none
- **Transport:** public HTTPS
- **Response format:** JSON
- **Polling:** rate limits should allow CoinGecko to poll at least once per minute
- **Response shape:** one row for every active, supported AarushDEX pool

Example response shape only. Values in angle brackets are placeholders, not market data:

```json
[
  {
    "ticker_id": "<BASE_MINT>_<TARGET_MINT>",
    "base_currency": "<BASE_MINT>",
    "target_currency": "<TARGET_MINT>",
    "last_price": "<TARGET_UNITS_PER_BASE_UNIT>",
    "base_volume": "<ROLLING_24H_BASE_VOLUME>",
    "target_volume": "<ROLLING_24H_TARGET_VOLUME>",
    "pool_id": "<POOL_ACCOUNT_PUBLIC_KEY>",
    "liquidity_in_usd": "<TOTAL_POOL_LIQUIDITY_USD>"
  }
]
```

### Ticker field mapping

| Field | CoinGecko requirement | AarushDEX implementation rule |
|---|---|---|
| `ticker_id` | Mandatory | Stable market identifier with base and target mint addresses separated by `_`. If multiple pools can exist for one mint pair, include a documented pool discriminator so each market is unique. |
| `base_currency` | Mandatory | Base token's Solana mint address. |
| `target_currency` | Mandatory | Target token's Solana mint address. |
| `last_price` | Mandatory | Price of the most recent completed swap, expressed as target-token units per one base-token unit. Do not substitute the reserve ratio unless CoinGecko explicitly accepts that as the reported price. |
| `base_volume` | Mandatory | Rolling 24-hour single-sided traded volume in base-token units. |
| `target_volume` | Mandatory | Rolling 24-hour single-sided traded volume in target-token units. |
| `pool_id` | Mandatory for DEX | Solana public key of the pool account. |
| `liquidity_in_usd` | Mandatory for DEX | Total pool value: base reserve × base USD price + target reserve × target USD price. |
| `bid` | Recommended | Omit until a documented AMM-equivalent quote is implemented and reconciled with the trading UI. |
| `ask` | Recommended | Omit until a documented AMM-equivalent quote is implemented and reconciled with the trading UI. |
| `high` | Recommended | Highest completed trade price over the rolling 24-hour window. |
| `low` | Recommended | Lowest completed trade price over the rolling 24-hour window. |

Token amounts must be converted from raw integer units using the actual decimals read from each mint. The DEX base_currency and target_currency values should be mint addresses, not ticker symbols alone.

For every completed swap, calculate the traded base amount and corresponding target amount once. Aggregate these into their respective 24-hour fields; do not double-count the same swap by adding input and output amounts as separate trades. Use on-chain transaction identity (for example, transaction signature plus instruction/event index) to distinguish trades.

### `GET /api/v1/coingecko/orderbook?ticker_id=<BASE_MINT>_<TARGET_MINT>&depth=100`

**Not implemented.** CoinGecko's supplied standard says the /orderbook endpoint is optional for an AMM DEX if liquidity_in_usd is provided in /tickers and the ±2% depth calculation formula is supplied. This endpoint is therefore not listed as a mandatory deliverable for AarushDEX's AMM model.

### `GET /api/v1/coingecko/historical_trades?ticker_id=<BASE_MINT>_<TARGET_MINT>&type=buy&limit=100`

**Not implemented.** Historical trades are optional for an AMM DEX under the supplied standard. If developed later, each trade must have a unique trade_id (a timestamp alone is not unique), price, base and target volumes, Unix timestamp in seconds, and type (buy or sell) as described in the supplied CoinGecko standard.

## AMM ±2% depth methodology (proposed; requires on-chain validation)

The supplied CoinGecko standard permits an AMM DEX to omit /orderbook only if the ticker response includes liquidity_in_usd and the exchange provides a formula for deriving ±2% depth.

AarushDEX's public Jupiter adapter uses constant-product quoting. The following fee-aware formula is a proposed methodology. **It depends on the deployed swap program retaining the full gross input in the pool reserves while only the fee-adjusted input determines the swap output.** The repository inspected contains the SDK quote math and IDL, but does not establish the deployed program's complete fee-transfer behavior. Confirm this assumption against the deployed on-chain instruction before publishing the formula.

Let:

- \(X\) = current human-unit base-token reserve.
- \(Y\) = current human-unit target-token reserve.
- \(P_0 = Y/X\) = current marginal price in target units per base unit.
- \(f = \mathrm{fee\_bps}/10{,}000\) = input fee fraction.
- \(USD_B\) and \(USD_T\) = trusted USD reference prices for the base and target assets.

For \(R > 1\), define the required gross-input fraction:

\[
T(R,f)=\frac{-(2-f)+\sqrt{(2-f)^2+4(1-f)(R-1)}}{2(1-f)}
\]

Subject to the fee-retention assumption above:

- **Price rises by 2%:** target-token gross input is \(Y \times T(1.02,f)\). The USD depth on that side is \(Y \times T(1.02,f) \times USD_T\).
- **Price falls by 2%:** base-token gross input is \(X \times T(1/0.98,f)\). The USD depth on that side is \(X \times T(1/0.98,f) \times USD_B\).

This models the gross input needed to move the marginal spot price from \(P_0\) to \(1.02P_0\) or \(0.98P_0\) under the stated constant-product and fee-retention assumptions. If the deployed program routes fees outside the pool, the equation must be adjusted. USD prices must come from a documented source and be consistent with the prices displayed on the AarushDEX trading interface. Do not publish the resulting depth values until the formula passes parity tests against actual on-chain swaps.

## Current implementation status and work required

The inspected public repository has:
- The AarushDEX Solana program ID and IDL.
- A Jupiter AMM adapter.
- An example mainnet pool account.
- Exact-in quote logic using constant-product pricing.

The inspected public repository does not provide:
1. A deployed public HTTPS API or API documentation site.
2. A complete index of all AarushDEX pools.
3. Rolling 24-hour completed-trade aggregation.
4. A reliable USD price source and total USD pool-liquidity calculation.
5. Public CoinGecko ticker, orderbook, or historical-trade routes.
6. Website/API parity checks for the required published market metrics.

Before any REST API URLs can be submitted, these items must be implemented, deployed, and tested. The ticker values must match the trading interface. All endpoints must return JSON without authentication and remain accessible at the polling frequency required by CoinGecko.

## DEX fork / factory / router section

| Field | Submission |
|---|---|
| **DEX fork type** | Custom Solana constant-product AMM. AarushDEX is not identified as a fork of a DEX in CoinGecko's currently published supported-fork list. |
| **Factory Address** | Not established from the inspected repository. The repository documents the AarushDEX program ID and pool PDAs, but does not identify a separate factory address. |
| **Router Address** | Not established from the inspected repository. The public SDK constructs swaps for the AarushDEX program; it does not document a separate router address. |
| **GeckoTerminal URL (if any)** | No AarushDEX GeckoTerminal listing URL was verified. |

**Important distinction:** The AarushDEX program ID is `H6UKMGJB2ge93xvcE2WyoLDWYKtyqAKXKGid4nath757`. It is included as the program identifier only. Do not label it as a factory or router address unless CoinGecko/GeckoTerminal confirms that this is the appropriate address for its listing workflow.

CoinGecko's current supported-fork article does not list AarushDEX. Its current Non-EVM Chain/DEX guidance says that DEXs not covered by the supported-fork list should provide the required adapters and submit a Non-EVM Network / DEX Addition request to GeckoTerminal. This is a separate integration route from submitting a CoinGecko exchange REST API. The appropriate route should be confirmed with CoinGecko before submitting.

## References

- [CoinGecko Integration API Standards supplied for this submission](https://docs.google.com/document/d/1v27QFoQq1SKT3Priq3aqPgB70Xd_PnDzbOCiuoCyixw)
- [AarushDEX Jupiter AMM SDK repository](https://github.com/aarushasoffical/aarushdex-jupiter)
- [CoinGecko — How to Add a New Exchange (updated 30 October 2026)](https://support.coingecko.com/articles/2005455880-how-to-add-a-new-exchange-on-coingecko?lang=en)
- [GeckoTerminal — DEX Forks supported by GeckoTerminal (updated 2 October 2026)](https://support.coingecko.com/articles/7376867784-dex-forks-supported-by-geckoterminal)
- [GeckoTerminal — How to integrate Non-EVM Chain/DEX (updated 2 October 2026)](https://support.coingecko.com/articles/1851457136-how-to-integrate-non-evm-chain-dex-on-geckoterminal)
