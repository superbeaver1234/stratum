# Engineering Backlog

Backlog order follows correctness and safety dependencies rather than feature visibility.

## Phase 0 — Research

- [x] Repository governance, architecture, docs skeleton and safety invariants.
- [x] NiceHash published extranonce-subscribe semantics.
- [x] BIP310 version-rolling baseline.
- [x] DigiByte current V4 MultiShield formula/constants extraction.
- [x] ESF AuxPoW activation, chain id, RPC, serializer and commitment validation extraction.
- [ ] Current NiceHash SHA256 verifier/live-session transcript and compatibility matrix.
- [x] DGB SHA256 algorithm id/header version encoding extraction.
- [ ] DGB current `getblocktemplate`/coinbase/submission semantics.
- [ ] Historical DGB V4 `nBits` deterministic vectors.
- [ ] ESF valid/invalid AuxPoW deterministic vectors and regtest round-trip.
- [ ] Headframe/Braiins/ViaBTC payout + Stratum telemetry matrix.
- [x] Pin exact DGB and ESF production release/tag/commit.
- [ ] DGB and ESF runtime daemon version/network startup validation contract.

## Phase 1 — Profitability-only / paper data plane

- [ ] `chain-rpc` typed JSON-RPC transport with redacted errors/timeouts.
- [ ] `DigiByteRpcClient` read adapter.
- [ ] `AuxChain` RPC abstraction + ESF read adapter.
- [ ] BTC baseline adapter and manual fallback baseline input.
- [ ] exchange adapter abstraction and executable bid VWAP.
- [ ] canonical sat/PH/day money/hashrate unit types — implementation committed; mark complete after Rust CI is green.
- [ ] profitability calculation and input snapshot persistence.
- [ ] paper decision engine with hysteresis/risk gates.
- [ ] PostgreSQL schema/migrations for Phase 1 inputs/outputs.
- [ ] Prometheus metrics and paper-mode logs.

## Phase 2 — BTC Stratum proxy

- [ ] Stratum V1 server/session parser with bounds/timeouts.
- [ ] NiceHash subscribe/configure/authorize/extranonce negotiation.
- [ ] BIP310 version rolling.
- [ ] collision-free extranonce allocator.
- [ ] warm `BTCUpstream` client and plugin interface.
- [ ] immutable job registry and late-share grace routing.
- [ ] integration miner + mock pool tests.

## Phase 3 — Native DGB SHA256

- [ ] exact block/target/compact/merkle primitives with vectors — implementation committed; mark complete after Rust CI is green.
- [ ] DGB GBT parser and coinbase builder.
- [ ] SHA256 job builder and local share validation.
- [ ] network-solution reconstruction and `submitblock`.
- [ ] regtest/testnet/integration evidence.

## Phase 4 — DGB + ESF AuxPoW

- [ ] generic compatible-tree coordinator.
- [ ] ESF slot/commitment/proof adapter.
- [ ] parent-only, aux-only and simultaneous solve tests.
- [ ] deterministic `submitauxblock` regtest proof.

## Phase 5 — Auto switching

- [ ] BTC↔DGB state machine and warm pipelines.
- [ ] enter/exit hysteresis, cooldown/residence/window/confidence gates.
- [ ] emergency BTC-only override and fault injection tests.
- [ ] switch-latency instrumentation and <500 ms acceptance criterion.

## Phase 6 — Additional Aux chains

- [ ] protocol research + adapter per child.
- [ ] compatibility/collision planner across children.

## Phase 7 — Predictor / backtesting

- [ ] exact MultiShield kernel vectors — integer kernel implementation committed; >=100 real historical vectors still required.
- [ ] stochastic 1/2/3/5 block simulation including self-impact.
- [ ] historical replay engine and threshold grid search.

## Phase 8 — Production hardening

- [ ] HA/recovery strategy.
- [ ] full accounting/reconciliation and realized PnL.
- [ ] authenticated control API.
- [ ] Grafana dashboards/alerts.
- [ ] operational runbooks and security review.
