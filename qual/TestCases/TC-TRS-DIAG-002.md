---
id: TC-TRS-DIAG-002
type: TestCase
testLevel: L3
status: draft
name: "Verify W080 — a Sequence diagram raises a finding for each SendAction/AcceptAction of its subject ActionDef not covered by an edge; covered diagrams are clean; draft-suppressed; gateable with --deny W080."
verifies:
  - REQ-TRS-DIAG-002
---

Verify the `Sequence` diagram send/receive completeness rule (`W080`, §22.4) by running
the binary against minimal model fixtures.

```gherkin
Feature: Sequence diagram send/receive completeness (W080)

  Scenario: W080 — subject SendAction not covered by any edge
    Given a Sequence diagram whose subject ActionDef has a SendAction
    And no edges entry references that SendAction
    When the tool validates the model
    Then a W080 finding is emitted

  Scenario: no W080 — every send/accept action is covered by an edge
    Given a Sequence diagram whose subject ActionDef has a SendAction
    And an edges entry references that SendAction by qualified name
    When the tool validates the model
    Then no W080 finding is emitted

  Scenario: W080 — SendAction nested in an IfAction then-branch is reached
    Given a Sequence diagram whose subject ActionDef has a SendAction inside an IfAction then-branch
    And no edges entry references that nested SendAction
    When the tool validates the model
    Then a W080 finding is emitted

  Scenario: W080 is draft-suppressed
    Given the uncovered-SendAction model but with the Sequence diagram status set to draft
    When the tool validates the model
    Then no W080 finding is emitted

  Scenario: --deny W080 promotes the warning to a gate failure
    Given the uncovered-SendAction model
    When the tool validates the model with --deny W080
    Then the tool exits non-zero
```
