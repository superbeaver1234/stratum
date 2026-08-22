# Architecture Decision Records

## ADR-001 — Rust production implementation

Decision: use Rust/Tokio for Stratum hot path and production services.

Reason: high concurrency, predictable memory ownership, strong binary/protocol types and no GC pauses in the share path.

Consequences: Python is limited to research/backtesting; unsafe code requires explicit justification and review.

## ADR-002 — BTC is the safety backend

Decision: any critical uncertainty/failure in DGB, AUX, market, predictor or decision inputs forces BTC SAFE MODE.

Reason: paid NiceHash hashrate must continue receiving usable work.

Consequences: safety routing overrides nominal profitability.

## ADR-003 — BTC fallback uses a warm upstream connection

Decision: keep at least one BTC Stratum upstream session continuously connected and job-synchronized.

Reason: switch latency must not depend on upstream reconnect/handshake.

Consequences: maintain upstream health, extranonce/job state and telemetry even while DGB is active.

## ADR-004 — Jobs are immutable routing records

Decision: bind each issued `job_id` to backend, template generation and difficulty/version state until its acceptance window expires.

Reason: switching must not misroute late shares or classify them against the new chain.

Consequences: a bounded job registry and explicit stale/grace semantics are required.

## ADR-005 — Canonical profitability unit is sat/PH/day

Decision: normalize route EV to satoshi/PH/day before decisions.

Reason: BTC is the baseline asset and USD conversion should not inject an unnecessary comparison dependency.

Consequences: USD/PH/day and BTC/EH/day are display metrics only.

## ADR-006 — Protocol source precedes implementation

Decision: consensus/protocol code is gated by upstream-source research and deterministic vectors.

Reason: AuxPoW, DigiByte MultiShield and Stratum extension errors can silently waste paid hashrate or create invalid blocks.

Consequences: unresolved protocol facts are explicit blockers, not TODO guesses in production code.
