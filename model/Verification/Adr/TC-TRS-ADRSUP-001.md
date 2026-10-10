---
type: TestCase
id: TC-TRS-ADRSUP-001
name: "ADR supersedes: resolution, cycles, status and breakdownAdr checks, show and links"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-model/tests/adr_supersession.rs
verifies:
  - REQ-TRS-ADRSUP-001
tags:
  - adr
---

```gherkin
Feature: ADR supersession

  Scenario: resolution
    Then supersedes accepts a string or a list, and an unresolved or non-ADR target is E320

  Scenario: cycles
    Then a self-supersession and a two-ADR cycle are E321

  Scenario: status
    Then superseding an ADR that is not superseded is W313, and a superseded target is clean

  Scenario: breakdownAdr
    Then a requirement citing a superseded ADR is W314, a draft requirement is not

  Scenario: show and links
    Then show prints supersededBy for the old ADR and links lists the supersedes edge
```
