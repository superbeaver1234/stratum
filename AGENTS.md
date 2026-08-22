# AGENTS.md

This file is mandatory operating policy for every coding agent working in this repository. Read it before changing code or documentation.

## Priority order

1. Correctness.
2. Do not strand or waste paid NiceHash hashrate.
3. BTC safe fallback.
4. Consensus correctness.
5. Profitability accuracy.
6. Low switching latency.
7. Observability.
8. Performance.
9. Extensibility.
10. UI.

## Architecture rules

- Rust is the production language for Stratum hot paths, consensus-adjacent serialization, routing, accounting and control services.
- Python is restricted to research, backtests, data analysis and auxiliary tooling.
- Keep protocol/domain logic independent from transport, database and vendor adapters.
- Parent chains, AuxPoW child chains, BTC pools and exchanges must use explicit adapter interfaces; never hard-code DGB+ESF as the only topology.
- Job routing is immutable: every issued `job_id` records its backend/template generation so late shares are validated and routed against the job that created them.
- BTC upstream stays warm in auto/DGB mode so emergency fallback does not depend on reconnect latency.
- Experimental/native backends may fail closed; ingress must fail over to BTC SAFE MODE rather than stop issuing work.
- Split routing must remain representable even while MVP policy is all-or-nothing.
- All profitability values use satoshi/PH/day as the canonical comparison unit; display conversions are secondary.
- Architecture changes require an ADR in `docs/DECISIONS.md`. Never silently change architecture.

## Protocol and consensus discipline

- Research the protocol/source first, then implement it.
- For consensus/protocol behavior, prefer canonical upstream source code and deterministic vectors over blog posts or assumptions.
- If observed daemon behavior conflicts with an assumption, stop the affected implementation, document the discrepancy, research upstream source, then amend architecture/tests.
- Endianness, compact target conversion, serialization and version-bit behavior must be explicit at API boundaries.

## Coding style

- Stable Rust toolchain; `rustfmt` and `clippy` clean before commit.
- Avoid panics in network/RPC hot paths. Return typed errors with safe context.
- Newtype monetary, hash, target, difficulty and hashrate units where unit confusion could cause accounting/consensus errors.
- No floating point for consensus target/hash comparisons. Profitability math must document precision/rounding.
- Structured logging only; do not log raw credentials or authentication headers.

## Testing rules

- Tests are mandatory for consensus-critical and accounting-critical code.
- Consensus-critical code requires unit vectors and, where possible, real historical/regtest vectors.
- Required focus: SHA256d, compact bits/target, difficulty, merkle roots, coinbase/extranonce, serialization, DGB templates, AuxPoW commitments/branches/submission, share validation and MultiShield.
- Integration tests must cover BTC↔DGB switching, grace-period late shares, parent/AUX solves and safety fallback on RPC/exchange failures.
- Fuzz parsers and untrusted Stratum/RPC serialization boundaries when practical.

## Git rules

For each completed logical block:

1. run tests;
2. run lint/format checks;
3. update relevant docs;
4. review `git diff` for secrets and accidental changes;
5. commit with a focused conventional-style message.

Do not squash unrelated work into one giant commit.

## Documentation rules

- After every completed substantial task update `docs/DEVLOG.md`.
- Record significant architectural decisions in `docs/DECISIONS.md`.
- Update `CHANGELOG.md` for user-visible changes.
- Keep protocol evidence and unresolved questions in the relevant `docs/*.md` file.

## Secrets

Never commit API keys, RPC passwords, wallet seeds, private keys, exchange secrets, NiceHash secrets or bearer tokens. Secrets belong in environment variables, Docker secrets, or a future vault-compatible provider. `.env.example` must contain placeholders only.
