---
id: REQ-TRS-BT-001
type: Requirement
name: "The default binding-time vocabulary is the ordered triad compile < load < runtime"
status: draft
reqDomain: software
verificationMethod: test
---

Absent any project declaration, the tool shall use the ordered vocabulary `compile < load < runtime`, with the meanings of [[REQ-TRS-PARAM-004]], as the set of valid binding-time stage names.

**Acceptance criteria:**
- With no `[bindingTimes]` in `.syscribe.toml`, the effective vocabulary is exactly `compile`, `load`, `runtime`, in that order.
- The three default names remain valid in every project, however it refines the vocabulary.
