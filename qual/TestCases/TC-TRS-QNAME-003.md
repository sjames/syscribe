---
id: TC-TRS-QNAME-003
type: TestCase
testLevel: L3
status: draft
name: "Verify that the qualified-name element segment is the filename stem, not the name: label."
verifies:
  - REQ-TRS-QNAME-003
---

Verify that the qualified-name element segment is the filename stem, not the `name:` label.

```gherkin
Feature: Element qualified-name segment comes from the filename stem

  Scenario: name: in frontmatter does not replace the filename stem
    Given a file Engine.md with name: InternalCombustionEngine in its frontmatter
    When the tool is invoked
    Then the element has qualified name Engine
    And no qualified name InternalCombustionEngine is reported
```
