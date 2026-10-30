---
id: REQ-TRS-BT-009
type: Requirement
name: "Binding-window ordering is checked by validate as well as feature-check"
status: draft
reqDomain: software
verificationMethod: test
---

The window-ordering checks of [[REQ-TRS-BT-008]] shall run under `validate` as well as `feature-check`, so a CI that runs only `validate` sees them.

**Acceptance criteria:**
- A mis-ordered pair is reported by `validate` and by `feature-check`, once each, with no duplicate within one command.
