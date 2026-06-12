---
id: TC-TRS-QNAME-003
type: TestCase
testLevel: L3
status: draft
name: "Verify that the name: field in element frontmatter overrides the filename stem."
verifies:
  - REQ-TRS-QNAME-003
---

Verify that the name: field in element frontmatter overrides the filename stem.

```gherkin
Feature: Element name override via frontmatter name:

  Scenario: name: in frontmatter replaces the filename stem
    Given a file model/Engine.md with name: InternalCombustionEngine in its frontmatter
    When the tool is invoked
    Then the element has qualified name InternalCombustionEngine

  Scenario: Absent name: uses the filename stem
    Given a file model/Engine.md with no name: field in its frontmatter
    When the tool is invoked
    Then the element has qualified name Engine
```
