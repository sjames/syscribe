---
id: REQ-TRS-DERIVE-005
type: Requirement
name: "Derive engine shall report formula parse errors as E501 and missing refs as W500"
status: draft
reqDomain: software
verificationMethod: test
---

- An unparseable formula string **shall** produce error **`E501`** ("derive formula parse error") and the field is left unevaluated.
- A formula that references a field absent on some collection elements coerces those absent values to 0 for numeric aggregates and emits warning **`W500`** (opt-in; suppressed by default).
- A formula referencing a nonexistent element via `elements["QName"]` emits **`E502`** ("derive: element 'QName' not found") and the field evaluates to null.

**Acceptance criteria:**

- `sum(children.nonExistentField)` evaluates to 0 and emits W500 if opted in.
- `elements["NonExistent::Thing"].someField` emits E502.
- An invalid formula like `sum(` emits E501.
