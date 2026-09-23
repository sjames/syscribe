---
id: TC-TRS-LINKTYPE-006
type: TestCase
testLevel: L3
status: draft
name: "Verify extends inherits base rules and reverse index, relax suppresses listed codes per type only, coverage=false withholds coverage, and mutation never rewrites into the base field."
verifies:
  - REQ-TRS-LINKTYPE-006
---

```gherkin
Feature: User-defined link types (TC-TRS-LINKTYPE-006)

  Scenario: a relaxed code is not raised for the extending link
  Scenario: an unrelaxed extending link raises the base rule
  Scenario: a built-in link is never relaxed
  Scenario: coverage=true counts toward the base coverage rule
  Scenario: coverage=false does not count toward coverage
  Scenario: relaxing E310 on a derivedFrom extension
  Scenario: mutation does not rewrite an extending link into the base field
```
