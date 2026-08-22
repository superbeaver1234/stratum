# Development Log

## 2026-08-22 — Repository bootstrap and Phase 0 research start

### Added
- repository governance and secret exclusions;
- architecture and module boundaries;
- Phase 0 protocol research skeleton;
- safety invariants and phased backlog.

### Changed
- none.

### Problems
- Current NiceHash public help pages are JS-heavy; exact live SHA256 marketplace handshake/extensions still require verifier/live-session evidence.
- DigiByte current difficulty behavior must be implemented from the current V4 source path, not simplified historical descriptions.
- ESF AuxPoW RPC is confirmed, but exact proof/commitment serializer still needs extraction from source/tests before coding.

### Decisions
- selected Rust for production hot path;
- BTC fallback remains warm at all times;
- immutable job-to-backend routing is a core invariant;
- Phase 0 research gates consensus/protocol implementation.

### Tests
- documentation-only bootstrap; no production code yet.

### Next
- finish NiceHash compatibility matrix;
- extract DigiByte V4 MultiShield constants/algorithm identification and historical vectors;
- extract ESF AuxPoW serializer/commitment test vectors;
- verify candidate BTC pool Stratum/FPPS interfaces;
- then begin Phase 1 profitability-only crates.

## 2026-08-22 — Extract NiceHash, DigiByte V4 and ESF AuxPoW protocol rules

### Added
- exact NiceHash extranonce-subscribe/set-extranonce lifecycle from the published specification;
- exact current DigiByte V4 MultiShield timing window, damping/clamps and per-algo adjustment structure;
- exact ESF `CAuxPow` field serialization order;
- exact ESF merged-mining magic, tree-size/nonce encoding and validation constraints.

### Changed
- AuxPoW architecture now delegates slot/index and serialization rules to child adapters.
- MultiShield predictor architecture now separates a deterministic Core-equivalent target kernel from stochastic forward simulation.
- job identity now records an extranonce epoch.

### Problems
- ESF uses `nChainIndex == merkle_nonce % merkle_size`, not the common Namecoin/Dogecoin LCG slot function. A generic hard-coded AuxPoW implementation would be invalid.
- Current NiceHash live SHA256 method ordering/masks/minimum difficulty still need verifier/live capture.
- DGB SHA256 algo/version encoding and current GBT semantics are not yet pinned.

### Decisions
- recorded ADR-007 for exact MultiShield reproduction before forecasting;
- recorded ADR-008 for per-child AuxPoW consensus adapters.

### Tests
- source-level research only; deterministic vectors/regtest proof are the next gate before consensus implementation.

### Next
- port ESF functional/unit AuxPoW vectors;
- extract DGB SHA256 algorithm/version/GBT behavior and historical `nBits` vectors;
- run current NiceHash verifier/live compatibility capture;
- complete Headframe/Braiins/ViaBTC protocol matrix.
