---
type: TestCase
id: TC-TRS-SUS-LINKS-008
name: "suspect accept --all-unbaselined onboards only links that have no baseline"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/suspect_links.rs
tags:
  - traceability
  - suspect-links
  - cli
verifies:
  - REQ-TRS-SUS-LINKS-008
---

Verifies the `suspect accept --all-unbaselined` onboarding mode: it baselines every
unbaselined link, leaves existing baselines (including outstanding suspect flags)
untouched, is idempotent, and rejects being combined with `--all`.

```gherkin
Feature: suspect accept --all-unbaselined

  Scenario: Every unbaselined link gains a baseline
    Given a model whose trace links have no baselines
    When syscribe -m <root> suspect accept --all-unbaselined is run
    Then every link with a resolvable target gains a traceBaselines entry
    And a subsequent validate emits no W090

  Scenario: An existing suspect flag is not cleared
    Given a link that already has a stale baseline (W090 would fire)
    And another link that has no baseline
    When syscribe -m <root> suspect accept --all-unbaselined is run
    Then the previously unbaselined link is baselined
    And the stale link is left unchanged and still suspect

  Scenario: The onboarding run is idempotent
    Given --all-unbaselined has already been run
    When it is run again immediately
    Then nothing further is baselined and no error is reported

  Scenario: --all and --all-unbaselined are mutually exclusive
    When syscribe -m <root> suspect accept --all --all-unbaselined is run
    Then the command reports a usage error and writes nothing
```
