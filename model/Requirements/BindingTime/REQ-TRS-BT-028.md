---
type: Requirement
id: REQ-TRS-BT-028
name: "params export --schema emits a JSON Schema for the values file"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

`params export --schema FILE` shall emit a JSON Schema for the same entries: type, unit (as `description`), `range:` (as `minimum`/`maximum`), `enumValues:` (as `enum`), the required flag, `default:` and `fallback:`.

**Acceptance criteria:**
- A schema generated for a Configuration validates the values file generated from it with a standard JSON Schema validator.
