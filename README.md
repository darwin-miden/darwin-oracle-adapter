# darwin-oracle-adapter

Pragma Oracle adapter for Darwin Protocol on Miden, with dynamic address resolution and a signed-attestation fallback.

See [`darwin-docs/m1-architecture-spec.md`](https://github.com/darwin-miden/darwin-docs/blob/main/docs/m1-architecture-spec.md) §8 for the full specification.

## Why an adapter

Two operational realities make a thin adapter layer necessary on Miden testnet:

1. **Pragma's oracle account address changes between testnet iterations.** Hardcoding it into every Darwin user account would mean every iteration breaks Darwin.
2. **Pragma can be down or stale.** The grant explicitly requires a fallback for the M1 dependency window.

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

Scaffold. The Rust types and WIT interface match the M1 spec; the MASM procedure bodies are stubbed pending the Miden v0.14 toolchain. Two unit tests assert that the bundled assets are non-empty and that the `ReadStatus` enum round-trips cleanly.

## License

MIT.
