---
type: TestCase
id: TC-TRS-SUS-LINKS-004
name: "Stale baseline emits W090; unbaselined links stay silent; --deny W090 gates"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/suspect_links.rs
tags:
  - traceability
  - suspect-links
verifies:
  - REQ-TRS-SUS-LINKS-004
---

Verifies the core validation behaviour: a changed target under a baselined link emits
W090, an unbaselined link produces nothing, W090 is gateable, and an unresolvable target
does not masquerade as a content mismatch.

```gherkin
Feature: Suspect-link validation (W090)

  Scenario: A changed target under a baselined link emits W090
    Given a source with a baselined link to target T
    And T's current projection hash differs from the stored baseline
    When syscribe -m <root> validate is run
    Then the output contains W090
    And the W090 message names the source, the target T, and the link kind

  Scenario: A matching baseline emits nothing
    Given a source with a baselined link to target T
    And T's current projection hash equals the stored baseline
    When validate is run
    Then no W090 is emitted for that link

  Scenario: An unbaselined link is silent
    Given a source with a trace link to target T and no baseline for T
    When validate is run
    Then no finding of any kind is emitted for that link

  Scenario: W090 is gateable in CI
    Given at least one suspect link exists
    When syscribe -m <root> validate --deny W090 is run
    Then the process exits with code 2

  Scenario: An unresolvable baselined target does not emit W090
    Given a baselined link whose target cannot be resolved
    When validate is run
    Then no W090 is emitted for that link
    And the existing unresolved-cross-reference handling applies instead
```
