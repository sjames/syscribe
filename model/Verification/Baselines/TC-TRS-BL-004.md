---
type: TestCase
id: TC-TRS-BL-004
name: "baseline create writes element and manifest and captures the git commit"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/baseline.rs
tags:
  - baseline
  - cli
verifies:
  - REQ-TRS-BL-004
---

Verifies `baseline create`: it seals the scope, writes the `Baseline` element and the lean
JSON manifest, derives identity, captures the HEAD commit, guards a dirty tree, refuses
collisions, and is review-aware for both suspect and unbaselined links.

```gherkin
Feature: baseline create

  Scenario: create writes element and manifest
    Given a model on a clean git tree
    When `baseline create --tag REL-2026-07 --approver "J. Roe"` is run
    Then a Baseline element is written under model/Baselines/
    And baselines/<id>.manifest.json is written under the git root with per-element hashes and the aggregate
    And the element's seal.aggregateHash equals the manifest aggregate

  Scenario: identity and label default from the tag
    Given create is run with no --id and no --name
    Then the id is derived to the BL-* grammar from the tag
    And the label defaults to the tag

  Scenario: create captures the HEAD commit
    Then the Baseline gitCommit equals the current HEAD

  Scenario: create refuses on an existing id, tag, or manifest
    Given a baseline with the target id/tag already exists
    When create is run
    Then it refuses and writes nothing

  Scenario: create guards a dirty working tree
    Given uncommitted changes in scope
    When create is run without --allow-dirty
    Then it refuses; with --allow-dirty it proceeds

  Scenario: create does not create the git tag
    When create records gitTag REL-2026-07
    Then no git tag is created by the tool

  Scenario: --require-reviewed refuses on a suspect link
    Given an in-scope trace link is suspect (W090)
    When create is run with --require-reviewed
    Then it refuses and writes nothing

  Scenario: --require-reviewed refuses on an unbaselined link
    Given an in-scope trace link has no baseline at all
    When create is run with --require-reviewed
    Then it refuses and writes nothing

  Scenario: The manifest embeds a readiness snapshot with real counts
    Given a fixture with a known number of validation warnings and in-scope types
    Then the manifest records the validation error/warning counts and per-type element counts equal to the fixture's
```
