# Phase 0 Research Plan

## NiceHash

- Capture current SHA256 pool-verifier session.
- Capture at least one rented SHA256 marketplace session in a controlled endpoint.
- Build compatibility matrix for method ordering, BIP310, extranonce subscription, difficulty and reconnects.
- Confirm current minimum difficulty/extranonce constraints by region/order type.

## DigiByte

- Pin current DigiByte Core commit/version used in production.
- Identify SHA256 algo ID/version encoding.
- Extract V4 difficulty formula and all active mainnet constants.
- Verify `getblocktemplate` fields and block-version requirements.
- Produce historical blocks/headers vector and reproduce next `nBits` exactly.
- Verify subsidy and coinbase maturity rules at current height.

## ESF

- Pin current ESF v29 commit.
- Extract AuxPoW activation/chain ID/RPC schema.
- Extract commitment parsing, aux merkle index algorithm and serialized proof layout.
- Port deterministic unit/functional vectors into language-neutral fixture notes.
- Prefer regtest proof before any mainnet submission.

## BTC pools

For Headframe, Braiins and ViaBTC:
- verify available payout mode and current terms;
- verify Stratum V1 endpoint semantics, version rolling, extranonce and difficulty behavior;
- determine how to measure realized FPPS/PPS payout per PH/day;
- keep adapter contract vendor-neutral.

## Exit criteria

Phase 0 is complete when all consensus-critical fields needed for Phase 1/2 are either confirmed with evidence or explicitly deferred without blocking those phases.
