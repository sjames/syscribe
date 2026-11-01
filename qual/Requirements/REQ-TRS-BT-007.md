---
id: REQ-TRS-BT-007
type: Requirement
name: "An unknown stage or a window that closes before it opens is an error"
status: draft
reqDomain: software
verificationMethod: test
---

A `bindingTime:` or `latestBindingTime:` naming a stage not in the effective vocabulary, or a `latestBindingTime:` earlier than `bindingTime:`, shall be an error on the declaring element: `E230` for an unknown stage (generalising [[REQ-TRS-PARAM-004]]) and `E750` for an inverted window. An unknown stage is dropped and the window is read from the remaining key.

**Acceptance criteria:**
- `bindingTime: bogus` is an error; `compile`/`load`/`runtime` are clean.
- `bindingTime: load` with `latestBindingTime: compile` is `E750`.
- An unknown `latestBindingTime:` on a parameter, or an unknown `bindingTime:` on a feature, is `E230`.
