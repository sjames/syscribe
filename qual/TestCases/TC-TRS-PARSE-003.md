---
id: TC-TRS-PARSE-003
type: TestCase
testLevel: L3
status: draft
name: "Verify that standard build and tool directories are excluded from discovery."
verifies:
  - REQ-TRS-PARSE-003
---

Verify that standard build and tool directories are excluded from discovery.

```gherkin
Feature: Ignored directories

  Scenario Outline: Standard excluded directories are not processed
    Given a model directory containing a valid element at the root
    And a .md file inside <excluded_dir>/
    When the tool is invoked against the model root
    Then the element inside <excluded_dir>/ does not appear in the element count

    Examples:
      | excluded_dir  |
      | target        |
      | .git          |
      | .github       |
      | node_modules  |
      | dist          |

  Scenario: Hidden files are ignored
    Given a model directory containing a file named .hidden.md
    When the tool is invoked against the model root
    Then .hidden.md is not processed as a model element
```
