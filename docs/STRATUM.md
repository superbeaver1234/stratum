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
  version_mask,
  acceptance_deadline
}
```

Switching publishes a fresh `mining.notify` with `clean_jobs=true`. Old jobs remain addressable for a configurable grace period. A late share is validated against its original job/backend and is never attributed to the new route.

## Version rolling

Implement BIP310 semantics exactly. The accepted mask is an intersection of miner/server masks. Once negotiated, `mining.submit` carries `version_bits`; reject bits outside the negotiated mask. Preserve parent-chain version bits not exposed for rolling.

## Difficulty

Start with controlled fixed difficulty suitable for NiceHash pool verification and high hashrate. Vardiff is a later policy feature, not required for protocol correctness. Difficulty changes become effective on the next job.

## Extranonce

Allocate collision-free extranonce namespaces per session. Extranonce changes must never create duplicate parent coinbases across live sessions/jobs. Exact NiceHash extranonce-subscription transcript remains a Phase 0 verification item.

## Switching latency target

With warm BTC upstream and warm DGB/AUX templates: publish a replacement job in <500 ms, preferably far lower. Measure decision-to-notify and notify-to-first-share separately.
