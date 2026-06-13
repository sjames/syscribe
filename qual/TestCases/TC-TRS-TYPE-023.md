---
id: TC-TRS-TYPE-023
type: TestCase
testLevel: L3
status: draft
name: "Verify gitlink/ref mismatch detection: W512 fires when a submodule peer's ref resolves to a different commit than the parent's recorded gitlink, --deny W512 gates CI, a ref matching the gitlink is silent, and a non-submodule sibling peer never emits W512."
verifies:
  - REQ-TRS-TYPE-023
---

Verify `W512` over a real git submodule: a peer whose `.syscribe.toml` `ref:` disagrees with
the parent's `.gitmodules` gitlink warns and gates under `--deny`; a matching ref is silent;
a non-submodule sibling peer never warns.

```gherkin
Feature: Submodule gitlink vs ref (§14, W512)

  Scenario: W512 — ref disagrees with the submodule gitlink
    Given a parent repo whose submodule gitlink pins c2 but whose repo ref is v1 (c1)
    When the tool validates the composing model
    Then a W512 finding names the repo and the gitlink/ref disagreement

  Scenario: --deny W512 gates CI
    Given the mismatched composition
    When the tool validates with --deny W512
    Then it exits non-zero

  Scenario: ref matching the gitlink is silent
    Given the repo ref changed to v2 (c2), matching the gitlink
    When the tool validates the composing model
    Then no W512 finding is emitted

  Scenario: non-submodule peer does not warn
    Given a peer that is a sibling checkout, not a submodule of the parent
    When the tool validates the composing model
    Then no W512 finding is emitted
```
