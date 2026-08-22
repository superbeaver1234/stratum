# Rust crate plan

Production crates will be introduced only when their Phase 0 protocol boundaries are verified.

Planned boundaries:
- `stratum-protocol`
- `stratum-server`
- `job-manager`
- `share-validator`
- `btc-upstream`
- `chain-rpc`
- `dgb-mining`
- `auxpow`
- `market`
- `profitability`
- `decision-engine`
- `accounting`
- `storage`
- `control-api`
- `app`

Avoid cyclic dependencies: protocol/domain crates must not depend on storage/control/vendor adapters.
