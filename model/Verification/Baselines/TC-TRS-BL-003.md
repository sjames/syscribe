---
type: TestCase
id: TC-TRS-BL-003
name: "Scope selection resolves whole-model, subtree, and filters"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/baseline.rs
tags:
  - baseline
  - scope
verifies:
  - REQ-TRS-BL-003
---

Verifies scope resolution: default whole-model, a package subtree, and the composable
`types` / `status` / `tags` filters, with the resolved membership recorded.

```gherkin
Feature: Baseline scope selection

  Scenario: Absent scope covers the whole model
    Given a create with no scope selector
    Then every model element is in scope

  Scenario: Package scope restricts to a subtree
    Given frozenScope.package = VehicleSystem::Powertrain
    Then only elements under that qualified-name prefix are in scope

  Scenario: Filters compose as AND
    Given frozenScope.types = [Requirement] and frozenScope.status = [approved]
    Then only approved requirements are in scope

  Scenario: Resolved membership is recorded
    Given any resolved scope
    Then the manifest enumerates exactly the in-scope elements the seal covers

  Scenario: Baseline elements are excluded from scope
    Given a whole-model scope and several existing Baseline elements
    Then no Baseline element appears in the resolved in-scope set

  Scenario: An empty resolved scope is refused
    Given filters that match no element
    When create is run
    Then it refuses and writes nothing rather than sealing an empty set
```
