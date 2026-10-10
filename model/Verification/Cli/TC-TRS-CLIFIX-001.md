---
type: TestCase
id: TC-TRS-CLIFIX-001
name: "behavioral-coverage label, links external targets and ls path scopes"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/cli_output_fixes.rs
verifies:
  - REQ-TRS-CLIFIX-001
tags:
  - cli
---

```gherkin
Feature: CLI output defects

  Scenario: behavioral-coverage header
    Given a model and no scope
    Then the header does not contain the text <model>

  Scenario: links shows external implementedBy
    Given a PartDef with implementedBy https://example.com/x and crates.io:serde@1.0.200
    Then links prints external for both and (unresolved) for none of them

  Scenario: ls path form
    Given a package Requirements::System with children
    When ls Requirements/System is run
    Then the children of Requirements::System are listed
    And an unknown path prints a did-you-mean hint when a near scope exists
```
