---
id: TC-TRS-PARSE-002
type: TestCase
testLevel: L3
status: draft
name: "Verify that the tool recursively discovers .md files in nested subdirectories."
verifies:
  - REQ-TRS-PARSE-002
---

Verify that the tool recursively discovers .md files in nested subdirectories.

```gherkin
Feature: Recursive directory walk

  Scenario: Elements in nested directories are discovered
    Given a model directory with elements at depth 1, 2, and 3
    When the tool is invoked against the model root
    Then the element count equals the total number of .md files with valid frontmatter across all depths

  Scenario: Non-.md files are ignored
    Given a model directory containing .txt and .yaml files alongside .md files
    When the tool is invoked against that directory
    Then only the .md files contribute to the element count
```
