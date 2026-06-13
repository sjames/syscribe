---
id: TC-TRS-TYPE-021
type: TestCase
testLevel: L3
status: draft
name: "Verify multi-repository composition: E510 circular, E511 missing path, E512 dangling cross-repo ref, E513 unknown alias, E514 unknown qname, E515 duplicate stable id, W510 unpinned repo; a valid composition resolves a cross-repo verifies cleanly; the repos list command."
verifies:
  - REQ-TRS-TYPE-021
---

Verify multi-repository model composition: `[repos]` loading, `repoImports:`, cross-repo
reference resolution, the `E510`–`E515`/`W510` rules, and the `repos` CLI.

```gherkin
Feature: Multi-repository model composition (§14)

  Scenario: valid composition with a cross-repo verifies is clean
    Given a model importing a pinned peer repo and a TestCase verifying a peer requirement
    When the tool validates the model
    Then none of E510–E515 or W510 are emitted
    And the cross-repo verifies raises neither E102 nor E512

  Scenario: E510 — circular repo import
    Given a peer repo that transitively imports back into the composing model
    When the tool validates the model
    Then an E510 finding is emitted

  Scenario: E511 — repo path missing and no ref
    Given a [repos] entry whose path is absent on disk with no ref
    When the tool validates the model
    Then an E511 finding is emitted

  Scenario: E512 — dangling cross-repo reference
    Given a verifies referencing a stable id present in no repo, with repos configured
    When the tool validates the model
    Then an E512 finding is emitted

  Scenario: E513 — repoImports names an unknown alias
    Given a repoImports entry whose repo is not in [repos]
    When the tool validates the model
    Then an E513 finding is emitted

  Scenario: E514 — repoImports qname not in the peer
    Given a repoImports entry whose qname does not resolve in the named repo
    When the tool validates the model
    Then an E514 finding is emitted

  Scenario: E515 — duplicate stable id across repos
    Given a stable id declared in both the local model and the peer repo
    When the tool validates the model
    Then an E515 finding is emitted

  Scenario: W510 — repo with no ref
    Given a [repos] entry with no ref
    When the tool validates the model
    Then a W510 finding is emitted

  Scenario: repos list command
    Given the valid composition
    When `repos list` is run
    Then it reports the peer alias, its path, and its ref
```
