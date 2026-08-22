# Profitability Model

Canonical comparison unit: **satoshi / PH / day**.

## BTC baseline

```text
BTC_EV = measured_or_expected_fpps_sats_per_ph_day
       - upstream_pool_cost
       - measured_reject_loss
```

Prefer measured payout/hashrate once statistically meaningful; retain theoretical hashprice as a cross-check.

## Native DGB

For expected hashes `H` over horizon `T`, use exact target/difficulty-derived block probability and expected block value. Convert expected DGB to BTC using executable bid-side VWAP for the expected liquidation amount, not last trade.

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

Report current EV and weighted projections for 1, 2, 3 and 5 DGB-relevant future blocks. Inputs include predicted difficulty, expected block timing, our added hashrate, market-liquidity decay assumptions and confidence.

## Decision hysteresis

Defaults are configuration, not constants:
- enter DGB edge: +8%
- exit DGB edge: +3%

Also enforce minimum residence time, cooldown, minimum predicted window, confidence and risk gates.

## Self-impact

Simulation must model `network_hash_before`, `our_hash`, accelerated block arrival and subsequent MultiShield response. Never reuse pre-entry network hashrate as though our rented hashpower had no effect.

## Reproducibility

Persist every model input, normalized unit, fee/slippage assumption, confidence score, output EV and decision reason with timestamps/source ages.
