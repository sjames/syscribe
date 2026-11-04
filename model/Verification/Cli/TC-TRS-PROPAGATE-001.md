---
type: TestCase
id: TC-TRS-PROPAGATE-001
name: "propagate agrees with configure, honours stages and is read-only"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/propagate.rs
verifies:
  - REQ-TRS-PROPAGATE-001
tags:
  - cli
---

```gherkin
Feature: live propagation

  Scenario: equals configure
    Then for each Configuration of the example models the forced and free sets equal configure's

  Scenario: forcing, clashes and stages
    Then requires and excludes force features, a clash names the choices and constraints, and a stage defers features bound elsewhere

  Scenario: read-only and surfaces
    Then the model directory is unchanged and the CLI, MCP tool and LSP request return the same payload
```
