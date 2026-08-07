---
id: TC-TRS-PLANITEM-005
type: TestCase
testLevel: L3
status: draft
name: "Verify PlanningItem evidence: ref:/path: entries resolve correctly, a rationale: waives one entry only, and remote paths skip the local check."
verifies:
  - REQ-TRS-PLANITEM-005
---

```gherkin
Feature: PlanningItem evidence:
  Scenario: a resolving ref: entry validates cleanly
    Given a PlanningItem with evidence: [{ref: <a real element>}]
    When the model is validated
    Then no evidence-ref error is raised for that item

  Scenario: a dangling ref: entry is rejected
    Given a PlanningItem with evidence: [{ref: <nothing real>}]
    When the model is validated
    Then an evidence-ref error is raised

  Scenario: the same dangling ref: with a rationale: is waived
    Given the same dangling ref: entry, now carrying its own rationale:
    When the model is validated
    Then no evidence-ref error is raised for that item

  Scenario: an existing local path: entry validates cleanly
    Given a PlanningItem with evidence: [{path: <a real local file>}]
    When the model is validated
    Then no evidence-path error is raised for that item

  Scenario: a missing local path: entry is rejected
    Given a PlanningItem with evidence: [{path: <a file that does not exist>}]
    When the model is validated
    Then an evidence-path error is raised

  Scenario: the same missing path: with a rationale: is waived
    Given the same missing path: entry, now carrying its own rationale:
    When the model is validated
    Then no evidence-path error is raised for that item

  Scenario: a remote-URI path: entry skips the local existence check
    Given a PlanningItem with evidence: [{path: <a remote URI>}]
    When the model is validated
    Then no evidence-path error is raised for that item

  Scenario: a waiver is per-entry, not blanket
    Given a PlanningItem with two broken evidence: entries, one carrying its own rationale: and one not
    When the model is validated
    Then only the entry with no rationale: raises an error
```
