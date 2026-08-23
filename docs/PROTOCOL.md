# Protocol Research

Status legend: **CONFIRMED**, **UNKNOWN**, **BLOCKING**.

## NiceHash SHA256 ingress

### Confirmed

- **CONFIRMED:** Bitcoin-family version rolling is standardized by BIP310 via `mining.configure`. Successful negotiation constrains the server/miner mask intersection and adds `version_bits` to `mining.submit`.
- **CONFIRMED:** NiceHash publishes an extranonce-subscribe extension: after `mining.subscribe`, a capable client may send `mining.extranonce.subscribe`; a supporting server may later send `mining.set_extranonce(extranonce1, extranonce2_size)`. The new extranonce becomes effective with the following `mining.notify`, even if that notify reuses the same job id.
- **CONFIRMED:** NiceHash pool verification exposes algorithm-specific difficulty/extranonce compatibility constraints. Historical SHA256 examples are not treated as current constants.

### Required ingress wire surface

Support and test:
- `mining.subscribe`
- `mining.extranonce.subscribe`
- `mining.authorize`
- `mining.configure`
- `mining.notify`
- `mining.set_difficulty`
- `mining.set_extranonce`
- `mining.submit`

### Unknown / must verify against current verifier and live rented session

- Exact ordering and optionality of `configure`, `subscribe`, `extranonce.subscribe` and `authorize` on current NiceHash SHA256ASICBoost sessions.
- Version-rolling mask/min-bit-count actually requested by current workers.
- Current minimum share difficulty and extranonce2-size constraints by marketplace region/order.
- Reconnect/retry cadence and behavior on server-side method errors.

No production Stratum behavior may hard-code these unknowns before current verifier/live-session evidence exists.

## DigiByte SHA256

### Pinned authority

Production mining semantics target DigiByte Core `v9.26.5`, commit
`05b50e229db5a3d1fb316c77f3f6c62efa879b96`. `docs/DAEMON_COMPATIBILITY.md`
is the compatibility authority. Earlier research against `develop` remains valid for
`GetNextWorkRequiredV4` because the pinned tag contains the same `src/pow.cpp` blob.

### Header and algorithm encoding

From the pinned `src/primitives/block.h` / `block.cpp`:

```text
ALGO_SHA256D          = 0
BLOCK_VERSION_ALGO    = 0x0f00
BLOCK_VERSION_SHA256D = 0x0200
```

`CBlockHeader::SetAlgo(ALGO_SHA256D)` ORs `0x0200` into the header version.
`CBlockHeader::GetAlgo()` recovers SHA256d when `(version & 0x0f00) == 0x0200`.
For SHA256d, `GetPoWAlgoHash()` is the ordinary Bitcoin-family double-SHA256 of
the serialized 80-byte header.

Accepted algorithm names include `sha`, `sha256`, and `sha256d`; production code
uses the canonical `sha256d` spelling.

### MultiShield V4

- **CONFIRMED:** V4 applies after height `1,430,000`.
- **CONFIRMED:** overall target spacing is 15 seconds; five active algorithm slots imply a nominal 75-second per-algorithm cadence.
- **CONFIRMED:** the global averaging interval examines 50 prior chain edges and targets `750` seconds.
- **CONFIRMED:** elapsed time is `MTP(tip) - MTP(tip-50)`, damped as `750 + (raw-750)/4`, then clamped to `690..870` seconds.
- **CONFIRMED:** the starting target is the most recent block of the requested algorithm.
- **CONFIRMED:** local adjustment is iterative 4% target scaling based on `height(prev_algo) + 5 - 1 - height(tip)`.
- **CONFIRMED:** integer operation order, powLimit clamp and compact encoding are consensus-relevant.

`DgbMultiShieldV4` derives both MTP values from raw timestamps. A normal replay
therefore needs at least 61 contiguous blocks; more history may be supplied to
find a previous SHA block after an abnormal gap.

### `getblocktemplate` SHA256d semantics on v9.26.5

- **CONFIRMED:** the RPC accepts the mining algorithm as its second positional argument. Production callers MUST use:

```text
getblocktemplate({"rules":["segwit"]}, "sha256d")
```

rather than relying on the daemon-wide `miningAlgo` default.

- **CONFIRMED:** omitting the `segwit` rule is rejected with `RPC_INVALID_PARAMETER`.
- **CONFIRMED:** the returned template includes `pow_algo_id` and `pow_algo`; for SHA256d they must be `0` and `sha256d`.
- **CONFIRMED:** the normal mutable set emitted by this source is `time`, `transactions`, `prevblock`.
- **CONFIRMED:** `noncerange` is `00000000ffffffff`.
- **CONFIRMED:** the response provides both compact `bits` and full numeric `target`; these must describe the same target.
- **CONFIRMED:** `coinbaseaux` is emitted. In the pinned implementation the local `aux` object is initialized empty and no mandatory bytes are added to it on the ordinary path.
- **CONFIRMED:** `coinbasevalue` is emitted; miners construct the coinbase unless a future/conditional template actually emits the documented optional `coinbasetxn`.
- **CONFIRMED:** a default witness commitment is emitted when the block template requires one.
- **CONFIRMED:** DigiDollar-aware miners may request the `digidollar-oracle` GBT rule. If the resulting coinbase has the oracle commitment, the response advertises `!digidollar-oracle` and returns `default_oracle_commitment`.
- **CONFIRMED:** v9.26.5 buries already-active Taproot/DigiDollar/AlgoLock deployments, so their old version bits are not a stable source of `vbavailable` version-rolling space.

### Block serialization and submission

- Header serialization is the six Bitcoin-family fields: little-endian `nVersion`, internal/wire 32-byte previous hash, internal/wire merkle root, little-endian `nTime`, `nBits`, `nNonce`.
- SHA256d PoW is over exactly that 80-byte header.
- The full block uses normal Bitcoin-family transaction/block serialization as implemented by the pinned Core source.
- `submitblock <hexdata> [dummy]` is the BIP22-style full-block submission path; the second compatibility argument is accepted and ignored.

### Remaining DGB gates

- **BLOCKING for historical proof:** capture at least 100 real SHA256d blocks and replay each block's `nBits` from its prior tip byte-for-byte.
- **BLOCKING for native mining:** capture a real v9.26.5 SHA256d GBT fixture and prove a reconstructed solved block through a test-chain/regtest `submitblock` path.

`tools/capture_dgb_consensus.py` is the reproducible external-environment capture utility. It refuses wrong daemon versions, non-mainnet or IBD state and records raw headers rather than explorer-derived difficulty floats.

## ESF / ElevenSeventyFive

Production AuxPoW semantics target ElevenSeventyFive Core `v29.1.0`, commit
`3a59832c3c105e65339252b5efe1b6a796f94641`.

- **CONFIRMED:** mainnet AuxPoW activation height is `31733`.
- **CONFIRMED:** mainnet AuxPoW chain ID is `1175`.
- **CONFIRMED:** `getauxblock <payout-address>` returns child hash, chain id, previous block, coinbase value, bits, height and target.
- **CONFIRMED:** `submitauxblock <hash> <auxpow>` submits a serialized AuxPoW proof.
- **CONFIRMED:** the child version is finalized with AuxPoW flag/chain-id bits before the child hash handed to the merged miner is computed.
- **CONFIRMED:** ESF's current chain-slot rule is `nChainIndex == merkle_nonce % merkle_size`; this is not the common Namecoin/Dogecoin LCG chain-id indexing rule.
- **CONFIRMED:** v29.1.0, not v29.0.0, is the supported production target because it corrects the ASERT activation anchor at height 31733.

## BTC upstream research

- **CONFIRMED:** Headframe currently advertises FPPS and publishes server-to-server mining/accounting API surfaces suitable for measuring realized hashrate/rejects/reward rate.
- Exact Stratum endpoint/session behavior remains deployment/account specific and must be integration-tested.
- Braiins and ViaBTC protocol/payout telemetry compatibility remains Phase 0 work.

## Research gate

Phase 1 may consume daemon/market/BTC baseline data while ingress-wire unknowns remain open. Phase 2 (real NiceHash proxy) is blocked on a current NiceHash verifier/live-session compatibility matrix. Phase 3 remains blocked until the real GBT/submit proof is captured. Phase 4 remains blocked on deterministic ESF AuxPoW daemon acceptance/rejection evidence.
