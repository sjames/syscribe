---
type: Requirement
id: REQ-TRS-BT-031
name: "Projection retains an element whose appliesWhen depends on a later-bound feature"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

When projecting at a stage, an element whose `appliesWhen:` references a feature or parameter whose window extends beyond that stage shall be retained and marked variant-retained, not removed.

**Acceptance criteria:**
- An element applying only when a `runtime`-window feature is on is retained when projecting a `compile` Configuration, and removed in a `runtime`-stage Configuration that turns the feature off.
