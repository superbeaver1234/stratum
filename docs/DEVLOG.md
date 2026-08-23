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

## 2026-08-23 — Pin production DGB and ESF daemon versions

### Added
- `docs/DAEMON_COMPATIBILITY.md` with exact production tags, commits and source authorities;
- DigiByte SHA256 algorithm id/version encoding evidence from the pinned release;
- explicit GBT algorithm-selection contract for future DGB adapters.

### Changed
- DigiByte production target is `v9.26.5`, not the preliminary `v8.26.2` candidate.
- ESF production target is `v29.1.0`, not `v29.0.0`.

### Problems
- ESF v29.0.0 and v29.1.0 are consensus-incompatible from AuxPoW/ASERT activation at height 31733 because v29.1.0 corrected the ASERT anchor. Supporting v29.0.0 would make deterministic AuxPoW/regtest evidence target the wrong chain.
- A real pinned DigiByte daemon is still needed to capture a canonical SHA256d GBT fixture and a solved `submitblock` round-trip.

### Decisions
- production daemon compatibility is fail-closed by exact supported release/commit unless an explicitly tested override is configured;
- future DGB GBT callers must pass `sha256d` explicitly rather than depend on the daemon-wide mining algorithm default.

### Tests
- GitHub ref comparison proved `DigiByte-Core/digibyte:v9.26.5` resolves to `05b50e229db5a3d1fb316c77f3f6c62efa879b96`;
- the pinned v9.26.5 `src/pow.cpp` blob is identical to the previously researched `develop` blob;
- GitHub ref comparison proved `1175Dev/1175:v29.1.0` resolves to `3a59832c3c105e65339252b5efe1b6a796f94641`.

### Next
- implement canonical integer/hash/target primitives;
- implement Core-equivalent MultiShield V4 kernel;
- capture historical DGB vectors;
- prove ESF AuxPoW round-trip against the pinned daemon.

## 2026-08-23 — Add canonical mining primitives

### Added
- dependency-free `stratum-primitives` Rust crate;
- explicit display-hex versus wire-little-endian `Hash256` APIs;
- exact 256-bit target representation and Bitcoin Core compatible compact-target conversions;
- integer target multiply/divide helpers for consensus kernels;
- SHA-256, SHA256d, 80-byte Bitcoin-family header serialization and Bitcoin merkle root;
- strongly typed money/hashrate/profitability units;
- deterministic vectors including Bitcoin genesis header and compact target examples.

### Changed
- root workspace now contains the first production crate.

### Problems
- the current execution container does not have `rustc` or `cargo`; local `cargo fmt`, `cargo clippy`, and `cargo test` cannot be executed here.
- correctness validation therefore moves to repository CI; no local pass is claimed.

### Decisions
- consensus primitives avoid IEEE floating point;
- the initial target/hash/crypto layer has no third-party dependencies, reducing supply-chain and offline-build surface for consensus-critical code.

### Tests
- test vectors are committed with the crate, but remain unverified until Rust CI executes them.

### Next
- add GitHub Actions for fmt/clippy/tests;
- resolve any compiler or vector failures before marking the primitives gate complete;
- implement the Core-equivalent MultiShield V4 kernel on top of these exact integer primitives.

## 2026-08-23 — Implement Core-equivalent MultiShield V4 kernel

### Added
- `stratum-dgb` crate with `DgbMultiShieldV4` pinned to DigiByte Core v9.26.5;
- exact mainnet V4 constants, powLimit, per-algo adjustment order and compact-target output;
- immutable raw header metadata model with explicit DigiByte algorithm identifiers;
- internal 11-block median-time-past derivation and contiguous-history validation;
- deterministic synthetic vectors for equilibrium, fast/slow clamps, hardening and easing steps.

### Changed
- deterministic V4 input now uses raw timestamps instead of trusting RPC-provided MTP; ADR-010 records the 61-block minimum history contract.

### Problems
- the required >=100 real historical DGB next-`nBits` vectors have not yet been captured; therefore the consensus gate remains open.
- Rust CI result is still required before compiler-level correctness is claimed.

### Decisions
- no stochastic predictor work is introduced;
- the kernel contains no RPC/database/network access and no floating-point consensus arithmetic.

### Tests
- synthetic expected `nBits` vectors are included but not reported as passing until CI runs;
- source operation order matches pinned `GetNextWorkRequiredV4`: MTP delta, `/4` damping, `690..870` clamp, target multiply/divide, iterative local 4% steps, powLimit clamp, compact encoding.

### Next
- obtain real historical mainnet vectors and require byte-for-byte matches;
- fix any CI failures before marking MultiShield implementation complete;
- then proceed to ESF deterministic proof implementation and Phase 1 RPC/data-plane work.

## 2026-08-23 — Pin DGB SHA256 GBT/header semantics and capture workflow

### Added
- stable v9.26.5 SHA256d header/version and GBT semantics in `docs/PROTOCOL.md` and `docs/RPC.md`;
- reproducible `tools/capture_dgb_consensus.py` fixture collector for real mainnet headers, SHA256d GBT, and >=100 actual SHA256d next-`nBits` vectors.

### Changed
- the former broad DGB GBT research item is now source-proven; a narrower integration gate remains for a real pinned-daemon GBT fixture and solved `submitblock` round-trip.

### Problems
- this execution environment cannot connect a local pinned `digibyted` nor issue arbitrary authenticated external RPC, so real historical vectors cannot be generated here.

### Decisions
- explorer difficulty numbers are not acceptable substitutes for raw `version`/`bits`/timestamp headers;
- the capture utility fails closed on wrong Core version, wrong network, IBD, or non-SHA256d template identity.

### Tests
- pinned source confirms `ALGO_SHA256D=0`, version bits `0x0200`, explicit GBT algorithm parameter, mandatory `segwit` rule, mutable set, target/bits fields, and BIP22-style `submitblock` behavior.

### Next
- run the capture utility against DigiByte Core v9.26.5 mainnet and commit sanitized fixtures;
- replay >=100 vectors byte-for-byte with `DgbMultiShieldV4`.

## 2026-08-23 — Add ESF AuxPoW primitives

### Added
- `stratum-auxpow` crate with an `AuxChain` consensus-variant boundary;
- ESF modulo slot rule, commitment encoder, merkle branch application, semantic serialized-parent-coinbase type, and exact `CAuxPow` field serializer;
- unit vectors mirroring the pinned ESF functional-test single-chain layout.

### Changed
- `docs/AUXPOW.md` now distinguishes source-equivalent serializer tests from daemon-proven acceptance and documents the exact external regtest procedure.

### Problems
- no ESF v29.1.0 regtest daemon is available in this execution environment;
- Rust compiler/CI results are still required before the crate itself is considered test-proven.

### Decisions
- no claim of ESF round-trip completion is made until bytes produced by our Rust serializer are accepted by pinned v29.1.0 `submitauxblock` and mutated proofs are rejected.

### Tests
- committed vectors cover commitment byte order, modulo slot behavior, serializer field order, hashBlock derivation, merkle branch direction, branch shape, and parent coinbase index zero;
- upstream `feature_1175_auxpow.py` independently proves the same protocol family against the daemon, but that does not substitute for testing our bytes.

### Next
- run our Rust-built proof through v29.1.0 regtest at activation height 200;
- persist valid/invalid sanitized fixtures under `tests/fixtures/esf/`.
