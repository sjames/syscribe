---
type: Requirement
id: REQ-TRS-BT-008
name: "A dependent's binding window may not close before the window of what it depends on opens"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

For a parameter, against the sibling parameters named in its `derivedFrom:` expression and its `bindTo:` target, and for a feature, against the features it `requires:`, the tool shall report an error when the dependent's window closes before the dependency's window opens; checked only when both ends declare a window. This generalises `E229`.

**Acceptance criteria:**
- A parameter whose window closes at `compile`, `derivedFrom` a sibling whose window opens at `load`, is an error; the reverse is clean.
- A feature that `requires:` a feature whose window opens after its own closes is an error.
- An edge with an unspecified end is not checked.
