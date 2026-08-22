# Changelog

All notable user-visible changes will be documented here.

## Unreleased

### Added
- Initial repository governance and Phase 0 architecture documentation.
- Research skeleton for NiceHash Stratum, DigiByte, ESF AuxPoW and profitability.
- Source-pinned protocol notes for NiceHash extranonce rotation, DigiByte V4 MultiShield and ESF AuxPoW serialization/commitment rules.
- Explicit research gates for live NiceHash compatibility, DGB native mining and ESF deterministic vectors.

### Changed
- AuxPoW architecture now treats chain slot/index/serialization behavior as child-adapter consensus logic rather than assuming one universal AuxPoW variant.
- MultiShield prediction is split into an exact Core-equivalent target kernel and a probabilistic forward simulator.

### Security
- Secret-bearing files and common key formats are excluded by `.gitignore`; configuration examples contain placeholders only.
