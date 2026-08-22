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

Decision: bind each issued `job_id` to backend, template generation and difficulty/version/extranonce state until its acceptance window expires.

Reason: switching or extranonce rotation must not misroute late shares or classify them against new work.

Consequences: a bounded job registry and explicit stale/grace semantics are required.

## ADR-005 — Canonical profitability unit is sat/PH/day

Decision: normalize route EV to satoshi/PH/day before decisions.

Reason: BTC is the baseline asset and USD conversion should not inject an unnecessary comparison dependency.

Consequences: USD/PH/day and BTC/EH/day are display metrics only.

## ADR-006 — Protocol source precedes implementation

Decision: consensus/protocol code is gated by upstream-source research and deterministic vectors.

Reason: AuxPoW, DigiByte MultiShield and Stratum extension errors can silently waste paid hashrate or create invalid blocks.

Consequences: unresolved protocol facts are explicit blockers, not TODO guesses in production code.

## ADR-007 — MultiShield predictor has a deterministic Core-equivalent kernel

Decision: implement DigiByte V4 next-target reproduction as a deterministic integer-only kernel that is tested independently from the stochastic forward simulator.

Reason: profitability forecasting may be probabilistic, but the difficulty rule itself is consensus behavior and must reproduce DigiByte Core exactly.

Consequences: historical vectors must match compact `nBits` byte-for-byte before predictor forecasts can influence routing.

## ADR-008 — AuxPoW slot/serialization rules belong to each child adapter

Decision: `AuxChain` owns its expected merkle slot function, commitment encoding, endianness constraints and proof serializer. `AuxPoWCoordinator` composes only compatible adapters.

Reason: current ESF uses `slot = merkle_nonce % merkle_size`, which differs from common Namecoin/Dogecoin chain-id/LCG slot selection.

Consequences: adding a child chain requires protocol extraction and compatibility tests; configuration with colliding/incompatible children is rejected before jobs are emitted.
