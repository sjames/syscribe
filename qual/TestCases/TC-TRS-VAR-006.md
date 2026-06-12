---
id: TC-TRS-VAR-006
type: TestCase
testLevel: L3
status: draft
name: "Verify transitive package appliesWhen: effective condition, E228 nesting/placement, W026, escapes."
verifies:
  - REQ-TRS-VAR-006
---

```gherkin
Feature: transitive package appliesWhen
  Scenario: a package's appliesWhen gates its whole subtree under --config
  Scenario: nested or forbidden appliesWhen declarations are E228; empty gated package is W026
  Scenario: external references into a gated subtree escape; internal references do not
```
