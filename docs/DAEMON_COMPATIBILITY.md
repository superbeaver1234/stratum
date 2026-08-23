# Supported Daemons

This file pins the daemon source trees that production-facing adapters and
consensus tests are allowed to target. A newer branch is research input only
until this file is deliberately updated.

## DigiByte

Release: DigiByte Core v9.26.5
Tag: `v9.26.5`
Commit: `05b50e229db5a3d1fb316c77f3f6c62efa879b96`
Network: DigiByte mainnet
Core version: `9.26.5`
Expected numeric `CLIENT_VERSION`: `9260500`
RPC protocol: Bitcoin-Core-style HTTP JSON-RPC; there is no independent
versioned "RPC protocol" negotiated by clients. Adapter compatibility is pinned
to the RPC shapes in this source tag.

Why this pin:
- the project brief's preliminary `v8.26.2` candidate is no longer current;
- the official DigiByte site identifies v9.26.5 as the latest release;
- v9.26.5 is the first post-DigiDollar-activation patch release and fixes the
  oracle startup scan while burying already-active deployments;
- the mining paths required by this project are present on the v9.26.5 tag.

Consensus sources:
- `src/pow.cpp`
- `src/kernel/chainparams.cpp`
- `src/consensus/params.h`
- `src/primitives/block.h`
- `src/primitives/block.cpp`

Mining/RPC sources:
- `src/rpc/mining.cpp`
- `src/node/miner.cpp`
- `src/node/miner.h`

Verified:
- `ALGO_SHA256D == 0`;
- SHA256d header algorithm bits are `BLOCK_VERSION_SHA256D == (2 << 8)`;
- algorithm mask is `BLOCK_VERSION_ALGO == (15 << 8)`;
- `CBlockHeader::SetAlgo(ALGO_SHA256D)` ORs the SHA256d version bits into
  `nVersion`;
- `CBlockHeader::GetPoWAlgoHash()` returns normal header `GetHash()` for
  SHA256d, i.e. the 80-byte header is double-SHA256 hashed;
- `GetAlgoByName` accepts `sha`, `sha256`, and `sha256d`;
- `getblocktemplate` accepts the mining algorithm as its second positional
  argument; production callers MUST pass `sha256d` explicitly instead of
  relying on the daemon's global default;
- GBT requires the `segwit` rule on this release;
- GBT reports `pow_algo_id` and `pow_algo`;
- for SHA256d the expected values are `0` and `sha256d`;
- `submitblock` remains the BIP22-style full-block submission path;
- `GetNextWorkRequiredV4` on `v9.26.5` is source-identical to the version
  previously researched on `develop` (same `src/pow.cpp` blob
  `f4ea823f3e0fc4d4a69dfb87a2a1d74dccb56865`).

Important v9.26.5 GBT note:
v9.26.5 buried Taproot, DigiDollar and AlgoLock. Once active, GBT exposes them
as active rules and no longer offers their former version bits through
`vbavailable`. This matters for future version-rolling integration even though
it does not change the V4 difficulty kernel.

Still to prove with deterministic fixtures:
- full real mainnet `getblocktemplate({"rules":["segwit"]}, "sha256d")` response;
- coinbase/`coinbaseaux`/`mutable` fields actually emitted by a pinned daemon;
- full solved-block reconstruction and `submitblock` on a test chain.

## ElevenSeventyFive / ESF

Release: ElevenSeventyFive Core v29.1.0
Tag: `v29.1.0`
Commit: `3a59832c3c105e65339252b5efe1b6a796f94641`
Network: ESF mainnet
Core version: `29.1.0`
AuxPoW activation height: `31733`
AuxPoW chain ID: `1175`

Why this pin:
- v29.0.0 introduced AuxPoW/ASERT at height 31733;
- v29.1.0 is a consensus maintenance release that re-anchors ASERT to block
  31732 so activation at 31733 starts from the intended legacy difficulty;
- v29.0.0 and v29.1.0 diverge from activation height 31733, therefore v29.0.0
  MUST NOT be used as the production target for this project;
- tag `v29.1.0` resolves exactly to commit
  `3a59832c3c105e65339252b5efe1b6a796f94641`.

Consensus/AuxPoW sources:
- `src/kernel/chainparams.cpp`
- `src/consensus/params.h`
- `src/primitives/block.h`
- `src/primitives/block.cpp`
- `src/pow.cpp`

Mining/RPC sources:
- `src/rpc/mining.cpp`

Verified:
- mainnet AuxPoW activation is height `31733`;
- chain ID is `1175`;
- `getauxblock <payout-address>` returns the child work record;
- `submitauxblock <hash> <auxpow>` submits the serialized proof;
- AuxPoW magic is `fa be 6d 6d`;
- tree size and merkle nonce are little-endian `uint32`;
- ESF slot rule is `merkle_nonce % merkle_size`, not the Namecoin/Dogecoin LCG;
- `CAuxPow` serialization order is pinned in `docs/AUXPOW.md`;
- parent work is checked against the ESF child target;
- v29.1.0 contains the corrected ASERT anchor immediately before activation.

Still to prove:
- deterministic valid and invalid serializer fixtures against v29.1.0;
- regtest `getauxblock -> build proof -> submitauxblock` acceptance.

## Compatibility policy

1. Production adapters MUST identify the connected daemon version at startup.
2. A version outside the supported pin is unhealthy by default until explicitly
   allowed by configuration and compatibility tests.
3. Consensus vectors are generated against the exact commits above.
4. Updating either pin requires source comparison, tests, DEVLOG entry and,
   when behavior changes, an ADR.
