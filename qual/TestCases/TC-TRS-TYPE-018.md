---
id: TC-TRS-TYPE-018
type: TestCase
testLevel: L3
status: draft
name: "Verify the native ReviewRecord element: E700–E705 structural rules, W700 (closed review with open item), W704 (uncovered requirement), and the reviews / review / reviews --coverage commands plus template ReviewRecord."
verifies:
  - REQ-TRS-TYPE-018
---

Verify recognition, validation, and the read-only CLI surface for `ReviewRecord`.

```gherkin
Feature: Native ReviewRecord element (§19)

  Scenario: well-formed ReviewRecord validates clean
    Given a valid ReviewRecord covering a requirement, with a recordedAt pointer
    When the tool validates the model
    Then none of E700, E701, E702, E703, E704, E705, W700 are emitted

  Scenario: E700 — missing required fields
    Given a ReviewRecord missing reviewType and reviews
    When the tool validates the model
    Then an E700 finding is emitted

  Scenario: E701 — bad id pattern
    Given a ReviewRecord whose id is not an RR-* id
    When the tool validates the model
    Then an E701 finding is emitted

  Scenario: E702 — bad status
    Given a ReviewRecord whose status is not open/closed/waived
    When the tool validates the model
    Then an E702 finding is emitted

  Scenario: E703 — bad reviewType
    Given a ReviewRecord whose reviewType is not in the enum
    When the tool validates the model
    Then an E703 finding is emitted

  Scenario: E704 — unresolved reviews entry
    Given a ReviewRecord whose reviews entry resolves to no element
    When the tool validates the model
    Then an E704 finding is emitted

  Scenario: E705 — bad item disposition
    Given a ReviewRecord with an items entry whose disposition is invalid
    When the tool validates the model
    Then an E705 finding is emitted

  Scenario: W700 — closed review with an open item
    Given a closed ReviewRecord with an open action item
    When the tool validates the model
    Then a W700 finding is emitted

  Scenario: W704 — uncovered requirement
    Given a model with a ReviewRecord and an approved requirement covered by no review
    When the tool validates the model
    Then a W704 finding is emitted

  Scenario: reviews command lists records
    Given the clean model
    When `reviews` is run
    Then the review record id appears in the output

  Scenario: reviews --coverage shows the cross-table
    Given the coverage model
    When `reviews --coverage` is run
    Then the uncovered requirement is shown

  Scenario: review command shows detail
    Given the clean model
    When `review RR-SW-001` is run
    Then the review type is shown

  Scenario: template ReviewRecord produces a skeleton
    When the ReviewRecord template is printed
    Then it contains `type: ReviewRecord`
```
