# stratum

Production-oriented control plane and Stratum gateway for rented SHA256 hashpower.

## Status

Phase 0 — protocol/consensus research and architecture. No production mining is enabled yet.

## Safety invariant

NiceHash ingress must always have a valid job. Any critical failure in experimental/native-chain paths forces **BTC SAFE MODE**.

## Planned flow

```text
NiceHash SHA256
      |
      v
Stratum Manager
  |          |
  | BTC      | native SHA256 parent
  v          v
FPPS pool   DigiByte block builder
                |
                +--> AuxPoW coordinator --> ESF / future child chains
```

## Repository layout

- `crates/` — Rust production components (introduced phase-by-phase after protocol research).
- `docs/` — protocol, consensus, RPC, profitability and engineering records.
- `config/` — non-secret configuration examples.
- `research/` — Python-only research/backtest utilities (future phases).

## Development phases

0. Research and protocol verification.
1. Profitability-only / paper calculations.
2. BTC Stratum proxy.
3. Native DigiByte SHA256 mining.
4. DigiByte + ESF AuxPoW.
5. Automatic BTC ↔ DGB switching.
6. Additional AuxPoW chains.
7. MultiShield prediction and historical backtesting.
8. Production hardening, observability and redundancy.

Read `AGENTS.md` before making any change.
