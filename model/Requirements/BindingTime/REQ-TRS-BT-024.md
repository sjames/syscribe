---
type: Requirement
id: REQ-TRS-BT-024
name: "The exported file is deterministic and stamped with its source"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

The exported file shall begin with a comment header naming the generator, the source Configuration and stage, and a `projection-hash` — the blake3 digest of the whole contract (Configuration, stage, and every entry's type, unit, range, enum, requiredness, fixedness, default, fallback, window and value), independent of the generator version and of comments, so it changes exactly when the contract does, and shall use sorted keys and stable formatting.

**Acceptance criteria:**
- Two exports of the same model are byte-identical.
- The header carries the Configuration id, stage and hash.
