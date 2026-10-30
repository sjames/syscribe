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

When projecting at a stage, an element whose `appliesWhen:` references a feature or parameter whose window extends beyond that stage shall be retained and marked variant-retained, not removed. The condition is evaluated three-valued: a feature the staged Configuration has not decided and whose window extends beyond its stage is unknown, not false, so an element is retained only when the open features genuinely decide its condition (`A and B` is removed when `A` is false whatever `B` becomes). `appliesWhen:` names features, so that is where retention applies; features, the only operands `appliesWhen:` has.

**Acceptance criteria:**
- An element applying only when a `runtime`-window feature is on is retained when projecting a `compile` Configuration, and removed in a `runtime`-stage Configuration that turns the feature off.
