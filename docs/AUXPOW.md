# AuxPoW Research

Production authority: ElevenSeventyFive Core `v29.1.0`, commit
`3a59832c3c105e65339252b5efe1b6a796f94641`.

## ESF facts confirmed from pinned source

- Mainnet AuxPoW activation height: `31733`.
- Mainnet AuxPoW chain ID: `1175`.
- AuxPoW version flag: bit 8 (`1 << 8`).
- Child chain id occupies version bits `[31:16]`.
- ESF uses `getauxblock` for work and `submitauxblock` for submission.
- Child block version and merkle root are finalized before the child hash returned to the miner is computed.
- AuxPoW proof is limited to 4096 serialized bytes; each merkle branch is limited to depth 30.
- The `getauxblock` submission path decodes `CAuxPow` using `TX_WITH_WITNESS` and rejects trailing bytes.

## Exact ESF `CAuxPow` serialization order

Pinned ESF source serializes:

```text
1. coinbaseTx            CMutableTransaction, TX_WITH_WITNESS stream
2. hashBlock             uint256 parent block hash
3. vMerkleBranch         vector<uint256>
4. nIndex                int32 Core serialization
5. vChainMerkleBranch    vector<uint256>
6. nChainIndex           int32 Core serialization
7. parentBlock           pure 80-byte parent header
```

`stratum-auxpow::EsfAuxPowProof::serialize` mirrors this field order. The
serialized parent coinbase transaction is represented by the semantic
`SerializedParentCoinbaseTx` type rather than an unlabelled byte vector.

The parent header embedded in the proof is deliberately a pure 80-byte header and cannot recursively carry AuxPoW.

## Exact ESF merged-mining commitment

ESF searches the parent coinbase `scriptSig` for exactly one magic prefix:

```text
fa be 6d 6d
```

It must be followed by:

```text
32 bytes  auxiliary merkle root in uint256/internal wire byte order
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

`stratum-auxpow::AuxChain` encodes this boundary; `EsfAuxChain` implements the
ESF modulo slot rule and commitment format.

For multiple simultaneous children, the coordinator must search tree size/nonce for collision-free slots across adapters and may combine only children whose commitment-root format is mutually compatible. Incompatible variants must be rejected as an invalid route configuration rather than silently producing bad proofs.

## Upstream deterministic reference

Pinned ESF ships `test/functional/feature_1175_auxpow.py`. Its valid single-chain
reference constructs:
- a parent coinbase whose scriptSig is exactly magic + child-hash LE + `size=1` LE + `nonce=0` LE;
- empty parent and chain branches;
- parent transaction index `0` and chain index `0`;
- a pure parent header solved against the child `nBits` target;
- `hashBlock == parentBlock.GetHash()`.

The functional suite then submits the serialized proof through both
`getauxblock(hash, auxpow)` and `submitauxblock(hash, auxpow)` and checks daemon
acceptance. It also exercises wrong child commitment, same parent/child chain id,
underworked parent PoW and malformed/oversized proofs.

Our Rust serializer/layout tests are source-equivalent to that format, but **this
repository does not claim the daemon round-trip is proven until the Rust-built
proof itself is accepted by a pinned v29.1.0 regtest daemon**.

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

## Remaining blocker: our serializer against daemon

Execution requirements to close the gate externally:

1. Build ESF Core `v29.1.0` at commit `3a59832c3c105e65339252b5efe1b6a796f94641` with functional/regtest support.
2. Start a clean regtest daemon; AuxPoW activates at regtest height `200`.
3. Mine to height `199`.
4. Call `getauxblock` with a valid `resf` payout address.
5. Feed returned child hash/bits into the Rust ESF proof builder/serializer.
6. Submit its exact hex with `submitauxblock` and require `true` plus block-height increment.
7. Repeat with mutations for wrong root, nonce, size, slot, parent branch/header, child hash, truncation and byte order; require daemon rejection.
8. Store sanitized request/result fixtures under `tests/fixtures/esf/`.

Until steps 1–8 use **our Rust-produced bytes**, the M0.5 AuxPoW round-trip item remains open.
