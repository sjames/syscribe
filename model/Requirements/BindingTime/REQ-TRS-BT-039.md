---
type: Requirement
id: REQ-TRS-BT-039
name: "Every binding-time field, code and command is documented and registered"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

The format spec §9.7, `prompts/create-model.md`, `docs/model-guide/variability.md` and the validation catalogue (`prompts/spec/validation.md` and `docs/validation/rules.md`) shall describe each new field, code and command, with a worked example under `examples/`.

**Acceptance criteria:**
- The catalogue tests (`catalogue_tests`, `validation_docs_catalogue`) pass for every code added.
- `examples/` contains a staged-configuration worked example that validates clean.
- Every new code appears in the catalogue, `docs/validation/rules.md` and spec §11.12; every new command has a help page; the variability guide, the authoring prompt and the example describe the feature.
