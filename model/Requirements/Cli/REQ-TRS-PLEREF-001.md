---
type: Requirement
id: REQ-TRS-PLEREF-001
name: "A reference to a variant-gated element is projected consistently and an Allocation inherits its endpoints' gates"
status: draft
reqDomain: software
reqClass: system
tags:
  - variability
---

Gating one element shall not cascade errors into every ungated element that merely lists it (GH #234, items 1–3).

## Behavior

- In the configuration lens (`validate --config`, `--all-configs`), a list-valued reference from an active element to an element that exists in the full model but is inactive in the variant is a traceability escape: **`W019`**, never the per-kind resolution error. This covers `ReviewRecord.reviews`, `Argument.supports`/`evidence`, `PlanningItem.evidence[].ref`, `ConfirmationMeasure.confirms`, `TestPlan.demonstrates`/`testCases`. The per-kind resolution errors (`E704`, `E855`, `E716`, `E851`, `E603`, `E601`) are dropped from the lens as the other 150%-model resolution codes already are, and still apply to a reference that resolves nowhere in plain `validate`. A mandatory single reference (`FaultTree.topEvent`, `E902`) keeps its error.
- An `Allocation` element is inactive in a variant when any endpoint named in `allocatedFrom:`/`allocatedTo:` that exists in the full model is inactive there (the AND of the endpoint gates); it is dropped from the projection and raises no escape finding.
- The help for `validate` states the reach of the three checks: plain `validate` sees no escapes, `--all-configs` checks the stored configurations, `feature-check --deep` proves it for all valid configurations.
