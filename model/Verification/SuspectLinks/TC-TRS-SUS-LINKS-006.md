---
type: TestCase
id: TC-TRS-SUS-LINKS-006
name: "suspect list reports suspect and unbaselined links, read-only and deterministic"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/suspect_links.rs
tags:
  - traceability
  - suspect-links
  - cli
verifies:
  - REQ-TRS-SUS-LINKS-006
---

Verifies the `suspect list` subcommand: it surfaces suspect links and (on demand)
unbaselined links, is deterministic, and never mutates the model.

```gherkin
Feature: suspect list

  Scenario: Suspect links are listed
    Given a source S with a baselined link to changed target T
    When syscribe -m <root> suspect list is run
    Then the output includes S, T, and the link kind

  Scenario: Unbaselined links are discoverable
    Given a source with a trace link that has no baseline
    When suspect list reports unbaselined links
    Then that link appears as a baseline candidate

  Scenario: Output is deterministic
    Given the same model
    When suspect list is run twice
    Then the two outputs are byte-identical

  Scenario: The command is read-only
    Given any model state
    When suspect list is run
    Then no model file is modified
```
