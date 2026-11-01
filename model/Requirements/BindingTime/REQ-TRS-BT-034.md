---
type: Requirement
id: REQ-TRS-BT-034
name: "Late-bound variability reaching a safety requirement needs covering verification"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-BT-000]
breakdownAdr: Decisions::BindingTimeADR
tags:
  - variability
  - binding-time
---

An opt-in, draft-suppressed, `--deny`-gateable rule shall flag a feature or parameter whose window extends to `load` or later, and that influences a requirement carrying a SIL/ASIL/CAL level through `appliesWhen:` or placeholders, unless a non-draft verifying `TestCase` declares, in an optional `coversVariability:` list (feature `FEAT-*` ids or qualified names, and `<feature>.<param>`), that it exercises the item's declared range or values (`W753`). A `coversVariability:` entry naming nothing is `W754`.

**Acceptance criteria:**
- A `runtime` parameter placed into an ASIL requirement's text with no covering test is flagged; adding a covering `TestCase` clears it.
- Models with no binding windows are unaffected.
