---
id: REQ-TRS-SEC-007
type: Requirement
name: "W039 shall extend to CAL3 CybersecurityGoals requiring an I2 confirmation measure"
status: draft
reqDomain: software
verificationMethod: test
---

Warning **`W039`** **shall** be extended to also cover `CAL3` `CybersecurityGoal` elements that lack an `I2` (or higher) `cybersecurity_assessment` `ConfirmationMeasure`, per ISO/SAE 21434 §15.9.

The existing CAL4→I3 check is unaffected. The new check fires when a CAL3 CybersecurityGoal has no CM with `independenceLevel: I2` or `I3` and `measureType: cybersecurity_assessment` in its `confirms:` list.

**Acceptance criteria:**

- A CAL4 CSG with no I3 CM → W039 (existing, unchanged).
- A CAL3 CSG with no I2 CM (only I1, or none) → W039.
- A CAL3 CSG with an I2 CM confirming it → no W039.
- A CAL3 CSG with an I3 CM confirming it → no W039 (I3 ≥ I2).
- A CAL2 CSG with no I1 CM → no W039 (CAL2 is not gated).
