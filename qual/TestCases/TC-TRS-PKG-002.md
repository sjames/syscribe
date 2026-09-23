---
id: TC-TRS-PKG-002
type: TestCase
testLevel: L3
status: draft
name: "Verify lint-docs flags an _index.md that hand-enumerates three or more of its own members (W103), advisory only."
verifies:
  - REQ-TRS-PKG-002
---

```gherkin
Feature: W103 hand-enumerated package members
  Scenario: an _index.md enumerating three members raises W103 and exits zero
  Scenario: two members, foreign ids, or a non-_index.md file raise nothing
```
