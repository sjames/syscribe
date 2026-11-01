---
id: REQ-TRS-BT-003
type: Requirement
name: "A malformed [bindingTimes] entry is a warning and is ignored whole"
status: draft
reqDomain: software
verificationMethod: test
---

A `[bindingTimes]` entry with a cyclic or unsatisfiable order, a duplicate name, an unknown neighbour, an unknown key, or the same position as another declared stage (two stages that would occupy the same gap, so have no defensible relative order — both are ignored and each warning names the other) shall raise a warning (code allocated at implementation) and shall be ignored whole; the remainder of the vocabulary shall stay in force, as `[linkTypes]` does with `W630`.

**Acceptance criteria:**
- A cyclic `after:`/`before:` pair is reported once and leaves the effective order unchanged.
- An unknown key is reported and the entry is ignored.
- Two stages both `after = "compile"` are both ignored, each warning naming the other; naming one as the other's `after:` places both.
