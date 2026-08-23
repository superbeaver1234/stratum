# Changelog

All notable user-visible changes will be documented here.

## Unreleased

### Added
- Initial repository governance and Phase 0 architecture documentation.
- Research skeleton for NiceHash Stratum, DigiByte, ESF AuxPoW and profitability.
- Source-pinned protocol notes for NiceHash extranonce rotation, DigiByte V4 MultiShield and ESF AuxPoW serialization/commitment rules.
- Explicit research gates for live NiceHash compatibility, DGB native mining and ESF deterministic vectors.
- Exact production daemon compatibility pins for DigiByte Core v9.26.5 and ElevenSeventyFive Core v29.1.0.
- Canonical Rust mining primitives for hashes, targets, compact bits, SHA256d, merkle roots, block-header serialization and mining/accounting units.

### Changed
- AuxPoW architecture now treats chain slot/index/serialization behavior as child-adapter consensus logic rather than assuming one universal AuxPoW variant.
- MultiShield prediction is split into an exact Core-equivalent target kernel and a probabilistic forward simulator.
- Production daemon upgrades are fail-closed until their consensus/RPC compatibility has been tested against the pinned versions.

### Security
- Secret-bearing files and common key formats are excluded by `.gitignore`; configuration examples contain placeholders only.
- Consensus primitives initially avoid third-party dependencies and floating-point target arithmetic.
