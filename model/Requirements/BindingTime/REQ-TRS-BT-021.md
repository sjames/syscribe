---
type: Requirement
id: REQ-TRS-BT-021
name: "The tool computes the source and stage of every effective value"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

For every effective feature selection or parameter value of a Configuration the tool shall determine its source — own binding, a `derivedFrom:` ancestor (naming it), a `subConfigurations:` tier, `bindTo:` propagation, a `derivedFrom:` parameter expression, the fixed `value:`, or the `default:` — and the stage at which it was bound.

**Acceptance criteria:**
- A value inherited from a base reports that base and its stage.
- A defaulted value reports `default`; a propagated value reports the `bindTo:` source.
