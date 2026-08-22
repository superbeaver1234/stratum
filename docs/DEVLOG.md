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
