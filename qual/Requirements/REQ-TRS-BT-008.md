---
id: REQ-TRS-BT-008
type: Requirement
name: "A dependent's binding window may not close before the window of what it depends on opens"
status: draft
reqDomain: software
verificationMethod: test
---

For a parameter, against the sibling parameters named in its `derivedFrom:` expression and its `bindTo:` target, and for a feature, against the features it `requires:`, the tool shall report `E229` when the dependent's window closes before the dependency's window opens (`latest(dependent) < earliest(dependency)`, so no choice of stage can satisfy the dependency); checked only when both ends declare a window. This generalises `E229` of [[REQ-TRS-PARAM-004]].

**Acceptance criteria:**
- A parameter whose window closes at `compile`, `derivedFrom` a sibling whose window opens at `load`, is an error; the reverse is clean.
- A feature that `requires:` a feature whose window opens after its own closes is an error.
- A parameter `derivedFrom` or `bindTo` a target whose window overlaps its own is feasible and silent.
- A parameter `bindTo` a system parameter that binds later is `E229`.
- An edge with an unspecified end is not checked.
