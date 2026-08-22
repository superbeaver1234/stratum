# Stratum V1 Design Notes

## Design goal

NiceHash remains connected to one endpoint while backend work changes internally.

## Session state

Each connection tracks:
- connection/session id;
- authorized worker identity;
- extranonce1 and extranonce2 size;
- negotiated extensions and version mask;
- current share difficulty and effective-from job generation;
- duplicate-share cache;
- worker statistics and last activity.

## Job registry

```text
job_id -> {
  backend_id,
  template_id,
  generation,
  issued_at,
  clean_jobs,
  difficulty_epoch,
  extranonce_epoch,
  version_mask,
  acceptance_deadline
}
```

Switching publishes a fresh `mining.notify` with `clean_jobs=true`. Old jobs remain addressable for a configurable grace period. A late share is validated against its original job/backend and is never attributed to the new route.

## Version rolling

Implement BIP310 semantics exactly. The accepted mask is an intersection of miner/server masks. Once negotiated, `mining.submit` carries `version_bits`; reject bits outside the negotiated mask. Reconstruct the header version as the fixed job version plus only the negotiated rolling bits. Preserve parent-chain version bits not exposed for rolling.

## NiceHash extranonce subscription

NiceHash's published extension states that a client may send:

```text
mining.extranonce.subscribe []
```

after a successful `mining.subscribe`. A supporting pool can then send:

```text
mining.set_extranonce [extranonce1, extranonce2_size]
```

The replacement extranonce becomes active when the next `mining.notify` arrives. The job must be treated as new work even if its textual job id is unchanged. Therefore our internal job identity includes an `extranonce_epoch`; external `job_id` alone is never sufficient to identify work.

The server parser must tolerate a client requesting this extension, but Phase 2 will not assume every current NiceHash worker sends it until confirmed against a live/verifier transcript.

## Difficulty

Start with controlled fixed difficulty suitable for current NiceHash verification and high hashrate. Vardiff is a later policy feature, not required for protocol correctness. Difficulty changes become effective on the next job generation.

## Extranonce allocation

Allocate collision-free extranonce namespaces per session. Extranonce changes must never create duplicate parent coinbases across live sessions/jobs. Allocation and rollover need deterministic tests across reconnects and split-routing buckets.

## Switching latency target

With warm BTC upstream and warm DGB/AUX templates: publish a replacement job in <500 ms, preferably far lower. Measure decision-to-notify and notify-to-first-share separately.
