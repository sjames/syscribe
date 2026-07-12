---
type: TestCase
id: TC-TRS-SUS-LINKS-005
name: "suspect accept writes and refreshes baselines and clears the flag"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/suspect_links.rs
tags:
  - traceability
  - suspect-links
  - cli
verifies:
  - REQ-TRS-SUS-LINKS-005
---

Verifies the `suspect accept` subcommand: single-link capture, `--all` bulk re-baseline,
formatting preservation, the not-referenced error, and that acceptance clears W090.

```gherkin
Feature: suspect accept

  Scenario: Accepting a link writes the target's current hash
    Given a source S with a trace link to target T and no baseline for T
    When syscribe -m <root> suspect accept S T is run
    Then S's traceBaselines gains an entry for T equal to T's current projection hash

  Scenario: Accepting overwrites a stale baseline and clears the flag
    Given S has a stale baseline for T (T changed, W090 would fire)
    When suspect accept S T is run
    And validate is run afterward
    Then no W090 is emitted for the S-to-T link

  Scenario: accept --all re-baselines every suspect link
    Given several suspect links across the model
    When syscribe -m <root> suspect accept --all is run
    Then every previously suspect link has a refreshed baseline
    And a subsequent validate emits no W090

  Scenario: Accepting preserves other frontmatter
    Given a source S with several frontmatter fields
    When suspect accept S T is run
    Then only the traceBaselines field is modified

  Scenario: Accepting a non-referenced target is an error
    Given S has no trace link to target X
    When suspect accept S X is run
    Then the command reports an error and writes nothing
```
