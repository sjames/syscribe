---
id: REQ-TRS-DERIVE-005
type: Requirement
name: "Derive engine shall report formula parse errors as E505 and unknown element references as E506"
status: draft
reqDomain: software
verificationMethod: test
---

- An unparseable formula string **shall** produce error **`E505`** ("derive formula parse error") and the field is left unevaluated.
- A formula that references a field absent on some collection elements coerces those absent values to 0 for numeric aggregates. (No warning is emitted for this; `W500` is the View `viewpoint:` warning and is never used by the derive pass.)
- A formula referencing a nonexistent element via `elements["QName"]` emits **`E506`** ("derive: element 'QName' not found") and the field evaluates to null.
- The derive codes (`E504` cycle — REQ-TRS-DERIVE-004, `E505`, `E506`) **shall** be disjoint from the Allocation resolution codes
  `E500`–`E503` (REQ-TRS-VAL-009): no code carries both a derive and an Allocation meaning (GH #127 — the derive pass previously
  emitted `E501`/`E502` and reserved `E500`).

**Acceptance criteria:**

- `sum(children.nonExistentField)` evaluates to 0.
- `elements["NonExistent::Thing"].someField` emits E506 (not E502).
- An invalid formula like `sum(` emits E505 (not E501).
- A model with both a derive error and an unresolved Allocation reference reports each under its own code.
