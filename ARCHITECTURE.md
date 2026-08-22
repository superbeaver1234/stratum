# Architecture

## System context

```text
                    +---------------------+
                    |      NiceHash       |
                    |  SHA256 hashpower   |
                    +----------+----------+
                               |
                         Stratum V1
                               |
                               v
                +---------------------------+
                |      STRATUM MANAGER      |
                | sessions / extranonce     |
                | jobs / shares / accounting|
                +---------+-----------+-----+
                          |           |
                  BTC MODE|           |DGB MODE
                          v           v
                +--------------+  +--------------------+
                | BTCUpstream  |  | ParentChainManager |
                | warm FPPS    |  | DGB block builder  |
                +--------------+  +----------+---------+
                                              |
                                     +--------+--------+
                                     | AuxPoWCoordinator|
                                     +---+----------+---+
                                         |          |
                                         v          v
                                       ESF       Aux N
```

## Modules

- `StratumServer`: TCP framing, JSON-RPC message parsing, protocol negotiation.
- `SessionManager`: authorization, worker/session identity, extranonce allocation, difficulty state.
- `JobManager`: immutable job registry, clean-job generations, late-share grace period.
- `ShareValidator`: reconstruct header, apply version rolling, SHA256d, duplicate/stale/share-target/network-target checks.
- `BTCUpstream`: pool-agnostic warm Stratum client with measured payout/reject/latency telemetry.
- `ParentChain`: native SHA256 parent abstraction; first implementation DigiByte.
- `DigiByteBlockBuilder`: GBT parsing, coinbase, merkle root, header, block submission.
- `AuxChain`: child RPC/template/submission abstraction.
- `AuxPoWCoordinator`: 1 parent + N children commitment tree/proof generation.
- `MarketPriceEngine`: executable bid-side VWAP and liquidity/staleness controls.
- `ProfitabilityEngine`: canonical sat/PH/day EV, current and forward projections.
- `DecisionEngine`: hysteresis, residence/cooldown/window/confidence/risk rules.
- `AccountingEngine`: expected and realized revenues, NiceHash cost and net PnL.
- `Storage`: PostgreSQL event/history persistence.
- `Metrics`: Prometheus metrics; Grafana dashboards.
- `ControlAPI`: authenticated status and management endpoints.

## Critical invariants

1. An issued job is immutable and names its backend/template generation.
2. Shares are never reinterpreted under the currently active backend; they are evaluated against their originating job.
3. Backend switching does not require a NiceHash reconnect.
4. A critical DGB/AUX/market/predictor/storage error cannot prevent BTC work from being served.
5. BTC fallback connectivity and recent jobs stay warm.
6. Consensus target comparisons use exact integer arithmetic.
7. Price decisions use executable liquidity, not last-trade price.
8. All decision inputs and outputs are reconstructible from persisted events.

## Concurrency model

Tokio-based services with bounded channels between ingress, job state, upstream clients and submission workers. The share hot path avoids database/network calls before local validation. Block candidates are submitted asynchronously after deterministic validation; share acknowledgements follow backend-specific correctness requirements.

## Routing model

Routing policy produces allocations by logical hash buckets, not connection reconnects. MVP may emit `{btc: 1.0}` or `{dgb: 1.0}`; API/data types must permit fractional allocations for future split routing.

## Safety state machine

```text
AUTO/BTC <-----> AUTO/DGB
   ^                |
   | critical error |
   +----------------+
        BTC_SAFE
```

`BTC_SAFE` overrides profitability. Recovery to AUTO requires healthy BTC, healthy candidate backend, fresh market data and configured confidence/risk gates.
