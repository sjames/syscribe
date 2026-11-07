---
type: TestCase
id: TC-TRS-ATTRREF-002
name: "An unresolvable value reference is E331 listing the available attributes"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/attr_refs.rs
verifies:
  - REQ-TRS-ATTRREF-002
tags:
  - references
---

```gherkin
Feature: E331

  Scenario: unknown element, attribute or no scalar value
    Given bodies citing `{{NoSuchThing.size}}`, `{{PolicydBounds.snapshotBytes}}` and `{{PolicydBounds.powerIn}}`
    When validate runs
    Then E331 reports the element that does not resolve
    And E331 for the unknown attribute lists snapshotFixedBytes among the available attributes
    And E331 reports that powerIn has no scalar value

  Scenario: template text and code are not references
    Given a body with `{{ user.name }}`, and references inside an inline code span and a non-gherkin fence
    Then validate reports no E331
```
