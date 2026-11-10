---
type: TestCase
id: TC-TRS-BIND-002
name: "bind refuses a violated parameter constraint and validate reports it"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/bind_timeline.rs
verifies:
  - REQ-TRS-BIND-002
tags:
  - cli
---

```gherkin
Feature: cross-parameter constraints on a binding

  Scenario: a binding that breaks a parameterConstraints entry
    Then bind exits 1, lists E221 once under the added errors and writes nothing

  Scenario: a satisfied or undetermined constraint
    Then the binding is written and the model still validates

  Scenario: a Configuration edited by hand to break the limit
    Then validate exits 1 with E221 naming the constraint
```
