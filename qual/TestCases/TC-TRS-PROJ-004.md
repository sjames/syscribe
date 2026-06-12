---
id: TC-TRS-PROJ-004
type: TestCase
testLevel: L3
status: draft
name: "Verify the global appliesWhen-implication guarantee (E227 / W020) with witness."
verifies:
  - REQ-TRS-PROJ-004
---

```gherkin
Feature: Global appliesWhen-implication guarantee
  Scenario: a violable structural edge is proven across all variants
    Given an always-active Part typedBy a PartDef appliesWhen Feat
    When running feature-check --deep
    Then E227 is reported naming the part and target
  Scenario: a holding implication is not flagged
    Given a Part appliesWhen Feat typedBy a PartDef appliesWhen Feat
    Then no E227 is reported for that edge
  Scenario: traceability edge is a warning
    Given an always-active TestCase verifying a Requirement appliesWhen Feat
    Then W020 is reported
```
