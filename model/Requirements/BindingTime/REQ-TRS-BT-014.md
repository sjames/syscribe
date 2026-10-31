---
type: Requirement
id: REQ-TRS-BT-014
name: "Values bound by an ancestor are closed and may only be narrowed"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

A selection or value bound by a `derivedFrom:` ancestor shall be closed: a descendant shall not rebind it to a different value (`E754`; restating the same value is harmless and silent). A Configuration narrows a still-open parameter with an optional `parameterNarrowing:` map — `<feature>.<param>` to a `lo..hi` range string or a list of enum values — which shall lie inside the parameter's declared `range:`/`enumValues:` and inside every ancestor's narrowing (`E755`, which also covers a malformed entry or one naming no parameter); a binding shall lie inside the narrowing in force along the chain (`E756`).

**Acceptance criteria:**
- A child that rebinds a parameter its base bound to a different value is `E754`; the same value is silent.
- A child that sets a feature selection its staged base decided otherwise is `E754`.
- A child whose `parameterNarrowing:` shrinks the range, and whose binding lies inside it, is clean; one that widens it is `E755`; a binding outside the narrowed range is `E756`. The same holds for an enum list.
