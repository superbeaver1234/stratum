# AuxPoW Research

## ESF facts confirmed from current source

- Mainnet AuxPoW activation height: `31733`.
- Mainnet AuxPoW chain ID: `1175`.
- AuxPoW version flag: bit 8 (`1 << 8`).
- Child chain id occupies version bits `[31:16]`.
- ESF uses `getauxblock` for work and `submitauxblock` for submission.
- Child block version and merkle root are finalized before the child hash returned to the miner is computed.
- AuxPoW proof is limited to 4096 serialized bytes; each merkle branch is limited to depth 30.

## Exact ESF `CAuxPow` serialization order

Current ESF source serializes:

```text
1. coinbaseTx            CMutableTransaction
2. hashBlock             uint256 parent block hash
3. vMerkleBranch         vector<uint256>
4. nIndex                int32-style Core serialization
5. vChainMerkleBranch    vector<uint256>
6. nChainIndex           int32-style Core serialization
7. parentBlock           pure 80-byte parent header
```

The parent header embedded in the proof is deliberately a pure 80-byte header and cannot recursively carry AuxPoW.

## Exact ESF merged-mining commitment

ESF searches the parent coinbase `scriptSig` for exactly one magic prefix:

```text
fa be 6d 6d
```

It must be followed by:

```text
32 bytes  auxiliary merkle root
4 bytes   merkle tree size, little-endian uint32
4 bytes   merkle nonce, little-endian uint32
```

Validation then requires:
- the parent `hashBlock` matches the supplied parent header hash;
- parent-chain version high 16 bits are not equal to child chain id `1175`;
- parent coinbase is transaction index 0 and its merkle branch reaches the parent header merkle root;
- `merkle_size` is a power of two in `1..2^30`;
- auxiliary branch length is exactly `log2(merkle_size)`;
- `nChainIndex < merkle_size`;
- `nChainIndex == merkle_nonce % merkle_size`;
- the computed auxiliary merkle root equals the 32-byte root committed in the coinbase.

For an AuxPoW ESF block, ESF tests the **parent header hash against the ESF child target** (`block.nBits`) and then validates the commitment. The parent header does not need to satisfy the DGB parent target for an ESF-only solve.

## Critical compatibility finding

ESF does **not** use the common Namecoin/Dogecoin LCG expected-index function keyed by chain id. Its slot function is simply:

```text
slot = merkle_nonce % merkle_size
```

Therefore merged mining must not have one hard-coded "standard AuxPoW" slot algorithm. Each `AuxChain` adapter owns:
- expected slot/index function;
- commitment/root byte-order rules;
- strict parent/child chain-id constraints;
- proof serialization;
- RPC submission format.

For multiple simultaneous children, the coordinator must search tree size/nonce for collision-free slots across the adapters and may combine only children whose commitment-root format is mutually compatible. Incompatible variants must be rejected as an invalid route configuration rather than silently producing bad proofs.

## Coordinator model

```text
AuxPoWCoordinator
  -> gather child candidates
  -> group by compatible commitment format
  -> choose tree size + nonce
  -> ask each adapter for expected slot
  -> reject collisions / enlarge tree / retry nonce
  -> build one auxiliary merkle root for compatible children
  -> return parent coinbase commitment + per-child proof context
```

## Remaining Phase 0 / Phase 4 blockers

- Port deterministic valid/invalid vectors from ESF unit/functional tests.
- Verify exact transaction witness serialization policy used by `submitauxblock` RPC hex.
- Build a regtest round-trip: `getauxblock -> construct parent coinbase/header -> serialize CAuxPow -> submitauxblock`.
- Before enabling additional child chains, extract their own slot, endian, commitment and serializer rules independently.
