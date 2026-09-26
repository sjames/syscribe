---
type: TestCase
id: TC-RL-005
name: Verify the library is synchronous, 3.10+ and stdlib-only
sourceFile: ../tests/test_ratelimit.py
status: active
testFunctions:
- function: test_imports_are_stdlib_only
  scenario: Only standard library modules are imported
- function: test_api_is_synchronous
  scenario: The API contains no coroutines
- function: test_declares_python_3_10_minimum
  scenario: The declared minimum Python version is 3.10
testLevel: L1
verifies:
- REQ-RL-008
---

## Test Procedure

Static checks on the source tree; no limiter is run.

```gherkin
Feature: Environment constraints

  Scenario: Only standard library modules are imported
    Given the parsed source of src/ratelimit.py
    When every imported top-level module is collected
    Then each one is in sys.stdlib_module_names

  Scenario: The API contains no coroutines
    Given the parsed source of src/ratelimit.py
    Then it contains no async function definitions and no await expressions

  Scenario: The declared minimum Python version is 3.10
    Given pyproject.toml
    Then its requires-python is ">=3.10"
```

