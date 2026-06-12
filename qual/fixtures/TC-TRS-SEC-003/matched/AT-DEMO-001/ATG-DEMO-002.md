---
id: ATG-DEMO-002
type: AttackTreeGate
gateType: AND
name: AND gate — extract bus key AND defeat freshness check
inputs:
  - ATS-DEMO-001
  - ATS-DEMO-002
---

Sequential path: both steps are required, so the feasibility is the min over the
inputs = min(high, low) = low (the chain is only as feasible as its hardest step).
