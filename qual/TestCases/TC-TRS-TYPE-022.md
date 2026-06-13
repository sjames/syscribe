---
id: TC-TRS-TYPE-022
type: TestCase
testLevel: L3
status: draft
name: "Verify peer-repository ref-drift detection: W511 fires when a peer HEAD drifts from its pinned ref, --deny W511 gates CI (exit 2), an in-sync peer is silent, and undeterminable drift (no git) does not emit W511."
verifies:
  - REQ-TRS-TYPE-022
---

Verify `W511` ref-drift detection over real git work trees: a drifted peer warns and gates
under `--deny`, an in-sync peer is silent, and a non-git peer is silent.

```gherkin
Feature: Peer-repository ref drift (§14, W511)

  Scenario: W511 — peer HEAD drifted from its pinned ref
    Given a peer git repo pinned to ref v1 whose HEAD has moved past v1
    When the tool validates the composing model
    Then a W511 finding names the repo and its configured ref

  Scenario: --deny W511 gates CI
    Given the drifted composition
    When the tool validates with --deny W511
    Then it exits non-zero

  Scenario: in-sync peer is silent
    Given the peer work tree checked out at the pinned ref v1
    When the tool validates the composing model
    Then no W511 finding is emitted

  Scenario: undeterminable drift does not warn
    Given a peer that is not a git work tree but is pinned to a ref
    When the tool validates the composing model
    Then no W511 finding is emitted
```
