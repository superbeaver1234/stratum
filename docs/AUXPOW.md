# AuxPoW Research

## ESF facts confirmed from current source

- Mainnet AuxPoW activation height: `31733`.
- Mainnet AuxPoW chain ID: `1175`.
- ESF uses `getauxblock` for work and `submitauxblock` for submission.
- Child block version includes the AuxPoW flag and chain ID before the child hash is computed.
- Pending AuxPoW templates are keyed by child hash inside ESF; the coordinator must submit the exact child hash returned for that template.

## Generic Namecoin/Dogecoin-style merged-mining model

For N compatible children, build an auxiliary merkle tree of child block hashes. The parent coinbase commits to the auxiliary merkle root and tree metadata. Each child proof contains:
- parent coinbase transaction;
- branch proving coinbase inclusion in the parent merkle root;
- parent block header;
- branch proving child hash inclusion in the auxiliary merkle root;
- child merkle index/tree metadata required by that chain's consensus serializer.

A single parent header can satisfy zero, one or many child targets independently of whether it satisfies the parent target.

## ESF items still requiring source-level extraction before implementation

**BLOCKING for Phase 4:**
- exact `CAuxPow` serialization field order used by ESF v29;
- exact merged-mining commitment magic/placement and byte order;
- child-hash byte order inside aux merkle computation;
- expected chain-index function and nonce/tree-size encoding;
- coinbase branch serialization/index conventions;
- strict parent chain-ID checks;
- deterministic vector from ESF functional/unit tests.

These will be copied as testable protocol facts from `src/primitives/block.*`, `src/pow.cpp` and `test/functional/feature_1175_auxpow.py`; do not implement from memory or a different child chain.

## Coordinator abstraction

`AuxPoWCoordinator` accepts `Vec<AuxCandidate>` and produces a parent coinbase commitment plus per-child proof context. Chain-specific adapters may override commitment/proof serialization when a child is not compatible with the common Namecoin-style format.
