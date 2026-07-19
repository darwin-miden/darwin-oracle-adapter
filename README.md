# darwin-oracle-adapter

Pragma Oracle adapter for Darwin Protocol on Miden, with dynamic address resolution and a signed-attestation fallback.

See [`darwin-docs/architecture-spec.md`](https://github.com/darwin-miden/darwin-docs/blob/main/docs/architecture-spec.md) §8 for the full specification.

## Why an adapter

Two operational realities make a thin adapter layer necessary on Miden testnet:

1. **Pragma's oracle account address changes between testnet iterations.** Hardcoding it into every Darwin user account would mean every iteration breaks Darwin.
2. **Pragma can be down or stale.** We need a fallback so Darwin keeps quoting NAV even when Pragma is unreachable.

The adapter holds the *current* Pragma address in its own storage and exposes a stable WIT interface (`wit/oracle.wit`) to the Darwin Protocol Account. The Darwin team can rotate the Pragma pointer via an administrative `update_pragma_address` call without redeploying user-facing accounts.

## Layout

```
darwin-oracle-adapter/
├── wit/oracle.wit             # WIT interface (price-feed)
├── asm/adapter.masm           # Miden component MASM source
├── src/
│   ├── lib.rs                 # Rust API
│   ├── feed.rs                # PriceQuote, ReadStatus
│   ├── address.rs             # Storage layout + OracleAddress
│   └── fallback.rs            # Signed-attestation types
```

## Status

Live on Miden testnet. The adapter resolves the Pragma oracle account
dynamically (Pragma rotates between testnet iterations), exposes a
stable WIT interface to the Darwin Protocol Account, and queries
on-chain median prices via foreign-account state proofs. The
`oracle_query_real` binary in `darwin-protocol` reads live medians
end-to-end in ~10s on testnet — see
[`darwin-docs/status.md`](https://github.com/darwin-miden/darwin-docs/blob/main/docs/status.md)
for the latest verified numbers (ETH/USD, BTC/USD, USDT/USD).
Eleven unit tests pass, covering the WIT round-trips, bundled
asset integrity, and the signed-attestation fallback path used
when Pragma is unreachable.

## License

MIT.
