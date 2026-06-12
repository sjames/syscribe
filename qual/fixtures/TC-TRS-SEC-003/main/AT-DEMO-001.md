---
id: AT-DEMO-001
type: AttackTree
name: Attack tree for TS-DEMO-001 — forged torque-request injection
threatRef: TS-DEMO-001
status: approved
---

Attack tree for ISO/SAE 21434 §15.7 attack path analysis of threat
TS-DEMO-001. The root is an OR gate (ATG-DEMO-001) over two alternative paths:

1. **Bypass message authentication (ATG-DEMO-002 — AND):** the attacker must
   both extract the bus key (ATS-DEMO-001, feasibility high) AND defeat the
   freshness/counter check (ATS-DEMO-002, feasibility low). As a sequential
   path the feasibility is the **min** of its steps = low.
2. **Replay a captured authenticated frame (ATS-DEMO-003, feasibility medium):**
   a single-step alternative.

As an OR of alternatives the root feasibility is the **max** of the two paths =
max(low, medium) = **medium**. The linked ThreatScenario declares `high`, so the
tool flags the mismatch with W035.
