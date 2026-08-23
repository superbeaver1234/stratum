# Chain RPC Research

## DigiByteRpcClient target surface

Production target: DigiByte Core `v9.26.5` / `05b50e229db5a3d1fb316c77f3f6c62efa879b96`.

Read-only/state:
- `getblockchaininfo`
- `getnetworkinfo`
- `getblockcount`
- `getbestblockhash`
- `getblockhash`
- `getblockheader`
- `getblock`
- `getnetworkhashps` only with algorithm semantics understood by the caller
- `getdifficulty` only as display/diagnostic data, never as the consensus target authority
- `getmempoolinfo`
- `getblocktemplate`

Submission, not part of Phase-1 production flow:
- `submitblock`
- optional BIP23 proposal validation through `getblocktemplate` where used deliberately

### SHA256d GBT call

The pinned daemon takes the algorithm as the second positional argument:

```json
{
  "method": "getblocktemplate",
  "params": [
    {"rules": ["segwit"]},
    "sha256d"
  ]
}
```

The adapter must reject a response unless:
- `pow_algo_id == 0`;
- `pow_algo == "sha256d"`;
- `bits` parses as a valid compact target;
- full `target` equals the target represented by `bits`;
- `previousblockhash`, `height`, `curtime`/`mintime`, version and transactions are structurally valid;
- source daemon is mainnet, not in IBD, and matches the supported version policy.

The current source emits mutable capabilities `time`, `transactions`, `prevblock` and `noncerange=00000000ffffffff`. `coinbaseaux` is present but currently empty on the ordinary v9.26.5 path. `default_witness_commitment` is conditional. DigiDollar-aware template construction is opt-in through rule `digidollar-oracle`; an actual oracle commitment is advertised as mandatory with `!digidollar-oracle` and exported as `default_oracle_commitment`.

Do not infer SHA256-specific consensus difficulty from generic floating-point `getdifficulty`. Template `bits`/target and exact header history are the authorities.

### `submitblock`

Pinned v9.26.5 follows the BIP22-style RPC:

```text
submitblock <hexdata> [dummy]
```

The second argument is accepted for BIP22 compatibility and ignored. A null result means accepted; rejection strings/errors must be preserved as submission telemetry. Production submission will not auto-retry until idempotency/duplicate-result semantics are explicitly handled.

## Historical fixture capture

`tools/capture_dgb_consensus.py` connects only to an explicitly configured RPC endpoint via environment variables. It verifies:
- numeric Core version `9260500`;
- `/DigiByte Core:9.26.5/` subversion;
- `chain == main`;
- `initialblockdownload == false`;
- SHA256d GBT identity.

It then stores raw header `version`, `bits`, timestamp and height data and derives test cases only from actual SHA256d blocks. For a SHA256d block at height `H`, the block's on-chain `nBits` is the expected SHA256d result computed from tip `H-1`.

Secrets are never written to the fixture.

## ESF AuxPoW RPC

Production target: ElevenSeventyFive Core `v29.1.0` / `3a59832c3c105e65339252b5efe1b6a796f94641`.

Confirmed mainnet interface:

```text
getauxblock <payout-address>
  -> hash, chainid, previousblockhash, coinbasevalue, bits, height, target

submitauxblock <hash> <serialized-auxpow>
  -> true on accepted block
```

ESF rejects handing out AuxPoW work before activation, while submission of already-solved pending work is deliberately not blocked by the node's connection/IBD work-request guard.

Phase 1 uses work-request/read data only. `submitauxblock` is permitted only in deterministic integration/regtest proof until real mining phases are authorized.

## Adapter requirements

- async transport with connection reuse;
- deadline/timeout per method;
- bounded response body;
- retry policy only for explicitly safe read-only calls;
- no implicit retry for block/proof submission;
- health state and last-success timestamp;
- sanitized transport/RPC error classes;
- no credentials in `Debug`, `Display`, traces or metrics;
- record daemon version/network/chain identity at startup;
- refuse mining when node is on wrong network, in IBD, or template provenance is inconsistent.
