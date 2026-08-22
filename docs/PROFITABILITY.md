# Profitability Model

Canonical comparison unit: **satoshi / PH / day**.

## BTC baseline

```text
BTC_EV = measured_or_expected_fpps_sats_per_ph_day
       - upstream_pool_cost
       - measured_reject_loss
```

Prefer measured payout/hashrate once statistically meaningful; retain theoretical hashprice as a cross-check. Vendor adapters expose accepted work, rejects/stales, payout rate and confidence rather than leaking pool-specific models into the engine.

## Native DGB

For expected hashes `H` over horizon `T`, use exact target-derived solve probability and expected block value. Convert expected DGB to BTC using executable bid-side VWAP for the expected liquidation amount, not last trade.

## MultiShield forward model

The active DigiByte V4 difficulty rule depends on both global recent timing and the requested algorithm's most recent block. A forward simulation must therefore evolve the complete multi-algo chain state, not just a scalar SHA256 difficulty.

State required for an exact next-target calculation includes:
- at least the latest 50 overall blocks needed for the V4 MTP window;
- algorithm identity for historical and simulated blocks;
- previous same-algorithm block and `nBits`;
- heights and timestamps/MTP inputs;
- exact V4 constants and integer arithmetic order.

Forecasts for 1/2/3/5 future blocks should separate:
1. deterministic Core-equivalent `next_target(state, algo)`;
2. stochastic scenario generation for which algorithm finds each next block and at what time;
3. our-hash self-impact on SHA256 arrival intensity;
4. EV aggregation/confidence across scenarios.

This separation lets unit tests prove consensus-equivalent target calculation independently of probabilistic forecasting.

## Aux children

Compute each child independently from its target, reward/maturity and executable BTC conversion. One parent hash trial contributes to all compatible children simultaneously.

```text
DGB_STACK_EV = DGB_EV + sum(AUX_EV)
             - exchange_fees
             - slippage
             - expected_stale_loss
             - operational_costs
```

## Forward EV

Report current EV and weighted projections for 1, 2, 3 and 5 future DGB-relevant blocks. Inputs include predicted difficulty, expected block timing, our added hashrate, market-liquidity decay assumptions and confidence.

## Decision hysteresis

Defaults are configuration, not constants:
- enter DGB edge: +8%
- exit DGB edge: +3%

Also enforce minimum residence time, cooldown, minimum predicted window, confidence and risk gates.

## Self-impact

Simulation must model `network_hash_before`, `our_hash`, accelerated SHA256 block arrival and subsequent MultiShield response. Never reuse pre-entry network hashrate as though our rented hashpower had no effect.

## Reproducibility

Persist every model input, normalized unit, fee/slippage assumption, confidence score, output EV and decision reason with timestamps/source ages. Consensus-derived targets use integer arithmetic; decimal/fixed-point policy for financial calculations will be documented before Phase 1 implementation.
