---
id: FTG-SIL-002
type: FaultTreeGate
name: FTG-SIL-002
gateType: AND
title: AND gate — both diverse software channels produce the same erroneous output
inputs:
  - FTE-SIL-001
  - FTE-SIL-002
probability: 1.0e-12
---

AND gate representing the probability that both diverse software channels independently produce the same erroneous conflict-check output. For this gate to be activated, Channel A (FTE-SIL-001) AND Channel B (FTE-SIL-002) must both simultaneously conclude that two conflicting routes do not conflict.

**Software diversity argument.** Channel A and Channel B are developed by different teams using different programming languages, different tool chains, and different compilers, following the EN 50128 diverse software development process. The probability of both channels making the same error on the same input is the product of their individual failure probabilities — 1.0 × 10⁻⁶ /h × 1.0 × 10⁻⁶ /h = 1.0 × 10⁻¹² /h — provided the independence assumption holds.

**Independence assumption.** The diversity argument depends on: (a) independence of design failures (different algorithms and implementations), (b) independence of random execution errors (separate processors, separate memory), and (c) absence of common-cause inputs (both channels receive identical but independently read inputs from the vital processor). The independence claim is substantiated by the EN 50128 diverse development evidence package (see ADR for software architecture).

**AND gate probability: 1.0 × 10⁻¹² /h** — five orders of magnitude below the SIL 4 threshold; negligible contribution to the top event.
