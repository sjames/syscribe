---
type: TestCase
id: TC-TRS-BL-009
name: "Baseline type and commands are documented for humans and LLMs"
status: active
testLevel: L1
sourceFile: repo:crates/syscribe/tests/baseline.rs
tags:
  - baseline
  - documentation
verifies:
  - REQ-TRS-BL-009
---

Verifies that the `baseline` command and `Baseline` type are documented consistently with the
other element types and commands.

```gherkin
Feature: Baseline documentation

  Scenario: baseline has a help page
    When `syscribe help baseline` is run
    Then a man page for create / verify / diff / list / show is printed

  Scenario: The BL-* id scheme is documented
    Given the project ID Scheme reference
    Then the BL-* pattern is described alongside the other stable-id types

  Scenario: The type is described for LLM authoring
    Given the model generation prompt surfaced via --agent-instructions
    Then the Baseline type, frozenScope, seal, and lifecycle are described
```
