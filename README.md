# AarushDEX Jupiter AMM SDK

Jupiter AMM integration for AarushDEX on Solana.

## AarushDEX

Program ID:

H6UKMGJB2ge93xvcE2WyoLDWYKtyqAKXKGid4nath757

Example mainnet pool:

7tojge8bQy7HdQmXLwEFZU33Jd3Wch3pNEdXrLK3aLPw

## Supported swaps

- Exact-in swaps
- Token A -> Token B
- Token B -> Token A
- Constant-product pricing
- AarushDEX fee calculation

Exact-out swaps are currently not supported.

## Jupiter integration

This repository implements Jupiter's `jupiter-amm-interface` `Amm` trait.

The adapter provides:

- Pool-state deserialization
- Pool state updates through `update()`
- Exact-in quoting through `quote()`
- Swap account-meta construction
- Both AarushDEX swap directions

Heavy pool deserialization is performed during `update()` so that `quote()` operates on cached state.

## Verification

The integration includes live mainnet parity tests against an AarushDEX pool.

Run:

```bash
cargo test
```
