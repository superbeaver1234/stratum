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

### Confirmed from current DigiByte Core `develop`

- **CONFIRMED:** DigiByte remains multi-algorithm; current-era blocks use `GetNextWorkRequiredV4` after height `1,430,000`.
- **CONFIRMED:** Mainnet overall target spacing is 15 seconds, with five-algorithm V4 target spacing `15 * 5 = 75` seconds per algorithm.
- **CONFIRMED:** V4 uses a 10-interval averaging target of `750` seconds and inspects `NUM_ALGOS * nAveragingInterval = 50` recent chain blocks.
- **CONFIRMED:** V4 computes elapsed time from median-time-past, damps deviation by `/4`, then clamps the effective window to `690..870` seconds (`-8%/+16%`).
- **CONFIRMED:** The starting target is the previous block of the requested algorithm. A local per-algorithm target adjustment of 4% is applied according to the distance between that block and the current tip.
- **CONFIRMED:** Current native mining should use `getblocktemplate`/BIP22-style block construction and `submitblock`, not legacy `getwork`.

### V4 reproduction rule

For algorithm `algo` at tip `last`:

```text
first = last - 50 chain blocks
prev_algo = last previous block using algo
raw = MTP(last) - MTP(first)
effective = 750 + (raw - 750) / 4
effective = clamp(effective, 690, 870)
new_target = target(prev_algo) * effective / 750
adjustments = height(prev_algo) + 5 - 1 - height(last)
```

Then apply the exact iterative 4% integer target adjustments used by Core, followed by `powLimit` clamping and compact-target encoding. Integer operation order and truncation are consensus-relevant and must be reproduced exactly.

### Open DigiByte items

- **BLOCKING for native DGB mining:** pin the production daemon release/commit and extract the exact SHA256 algorithm id/version encoding used by current templates/headers.
- **BLOCKING for native DGB mining:** verify current `getblocktemplate` fields, coinbase/version requirements and SHA256-specific template selection behavior.
- **BLOCKING for predictor completion:** produce historical block vectors that reproduce next SHA256 `nBits` byte-for-byte.

## ESF / ElevenSeventyFive

- **CONFIRMED:** Current ESF Core is SHA-256 and activates AuxPoW at mainnet height `31733`.
- **CONFIRMED:** mainnet AuxPoW chain ID is `1175`.
- **CONFIRMED:** `getauxblock <payout-address>` returns child hash, chain id, previous block, coinbase value, bits, height and target.
- **CONFIRMED:** `submitauxblock <hash> <auxpow>` submits a serialized AuxPoW proof.
- **CONFIRMED:** the child version is finalized with AuxPoW flag/chain-id bits before the child hash handed to the merged miner is computed.
- **CONFIRMED:** ESF's current chain-slot rule is `nChainIndex == merkle_nonce % merkle_size`; this is not the common Namecoin/Dogecoin LCG chain-id indexing rule.

## BTC upstream research

- **CONFIRMED:** Headframe currently advertises FPPS and publishes server-to-server mining/accounting API surfaces suitable for measuring realized hashrate/rejects/reward rate.
- Exact Stratum endpoint/session behavior remains deployment/account specific and must be integration-tested.
- Braiins and ViaBTC protocol/payout telemetry compatibility remains Phase 0 work.

## Research gate

Phase 1 may consume daemon/market/BTC baseline data while ingress-wire unknowns remain open. Phase 2 (real NiceHash proxy) is blocked on a current NiceHash verifier/live-session compatibility matrix. Phase 3 is blocked on DGB SHA256 header/template semantics. Phase 4 is blocked on deterministic ESF AuxPoW vectors.
