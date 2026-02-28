# Methodology

This lab reproduces the *structure* of latency-sensitive trading claims in a controlled sandbox.

## Assumptions
- External signal is synthetic stochastic process.
- Market feed is lagged copy of external feed.
- Orders always fill at observed market price (simplified).
- Costs represented as basis-point fee only.

## Limitations
- No real market microstructure
- No queue priority / matching-engine dynamics
- No adversarial participants
- No legal/regulatory execution inferences

Use this to study **risk mechanics**, not to deploy real trading systems.
