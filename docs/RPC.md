# Chain RPC Research

## DigiByteRpcClient target surface

Read-only/state:
- `getblockchaininfo`
- `getblockcount`
- `getblockhash`
- `getblockheader`
- `getnetworkhashps` where semantics are verified for the selected algorithm
- `getblocktemplate`
- mempool/template fee data needed for reward EV

Submission:
- `submitblock`
- optional BIP23 proposal validation through `getblocktemplate` where supported

Do not infer SHA256-specific difficulty from a generic scalar RPC without verifying current DigiByte response semantics. Prefer template `bits`/target plus source-consistent algo history.

## ESF AuxPoW RPC

Current ESF mainnet source confirms:

```text
getauxblock <payout-address>
  -> hash, chainid, previousblockhash, coinbasevalue, bits, height, target

submitauxblock <hash> <serialized-auxpow>
  -> true on accepted block
```

ESF rejects handing out AuxPoW work before activation, while submission of already-solved pending work is deliberately not blocked by the node's connection/IBD work-request guard.

## Adapter requirements

- deadline/timeout per method;
- retry policy only where idempotent/safe;
- health state and last-success timestamp;
- sanitized error classes;
- no credentials in logs;
- record daemon version/network/chain identity at startup;
- refuse mining when node is on wrong network, in IBD, or template provenance is inconsistent.
