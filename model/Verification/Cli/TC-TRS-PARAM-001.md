---
type: TestCase
id: TC-TRS-PARAM-001
name: "constraints check evaluates constraints against attribute values, defaults, configuration parameters and overrides"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/constraints_check.rs
verifies:
  - REQ-TRS-PARAM-001
tags:
  - cli
---

```gherkin
Feature: parametric constraint checking

  Scenario: pass, fail, unresolved, error
    Then a satisfied constraint passes, a violated one fails (exit 2), a missing input is unresolved (exit 0, 2 with --strict) and a bad expression is an error

  Scenario: binding and typing
    Then defaults apply, bindings may be expressions, ScalarValues parameters drop units and ISQ parameters are dimension-checked

  Scenario: requirement clauses and negation
    Then requires/assume clauses are checked and isNegated inverts

  Scenario: calculations
    Then calc evaluates a CalculationDef body with defaults and --set, and refuses non-calculations and opaque languages

  Scenario: sweeps
    Then sweep evaluates the checks over the grid of parameter ranges with optional shown expressions, and rejects bad ranges

  Scenario: configurations and overrides
    Then --config uses its parameter bindings, --all-configs tabulates, --set wins and an unknown configuration exits 1
```
