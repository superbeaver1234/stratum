# Protocol Research

Status legend: **CONFIRMED**, **UNKNOWN**, **BLOCKING**.

## NiceHash SHA256 ingress

### Confirmed

- **CONFIRMED:** Hashpower Marketplace pool targets are tested by NiceHash and must satisfy algorithm-specific minimum difficulty/extranonce constraints.
- **CONFIRMED:** Bitcoin-family version rolling is standardized by BIP310 via `mining.configure`; successful negotiation adds the sixth `version_bits` parameter to `mining.submit`.
- **CONFIRMED:** NiceHash historically documents extranonce subscription (`#xnsub`) for SHA256 ASIC routing. This is evidence that extranonce changes are operationally relevant, not permission to assume every marketplace session uses a particular extension transcript.

### Required wire surface

Support and test:
- `mining.subscribe`
- `mining.authorize`
- `mining.configure`
- `mining.notify`
- `mining.set_difficulty`
- `mining.set_extranonce` / extranonce subscription where negotiated
- `mining.submit`

### Unknown / must verify against current pool verifier and live rented session

- Exact ordering and optionality of `configure`, `subscribe`, `authorize` in current NiceHash SHA256 marketplace connections.
- Whether current NiceHash ingress sends `mining.extranonce.subscribe` or only expects server-side behavior inferred from pool verification.
- Version-rolling mask/min-bit-count actually requested by current SHA256ASICBoost workers.
- Current minimum share difficulty and extranonce2-size constraints by marketplace region/order.
- Reconnect behavior and retry cadence.

No production Stratum behavior may hard-code these unknowns before packet/transcript capture or official current evidence.

## DigiByte

- **CONFIRMED:** Current DigiByte Core retains multi-algorithm mining with a dedicated SHA256D algorithm and per-block difficulty logic.
- **CONFIRMED:** Current `develop` source routes current-era difficulty through `GetNextWorkRequiredV4` after the configured activation height.
- **CONFIRMED:** `getblocktemplate`/BIP22-style block construction is the intended modern mining workflow; no design dependency on legacy `getwork`.

## ESF / ElevenSeventyFive

- **CONFIRMED:** Current ESF Core is SHA-256 and activates AuxPoW at mainnet height 31733.
- **CONFIRMED:** mainnet AuxPoW chain ID is 1175.
- **CONFIRMED:** `getauxblock` returns child hash, chainid, previous block, coinbase value, bits, height and expanded target; `submitauxblock(hash, auxpow)` submits a serialized proof.
- **CONFIRMED:** ESF sets AuxPoW version/chain-id bits before computing the child hash handed to miners.

## Research gate

Phase 1 may consume RPC/market data before all ingress unknowns are closed. Phase 2 (real NiceHash Stratum proxy) is blocked on a current NiceHash verifier/live-session compatibility matrix.
