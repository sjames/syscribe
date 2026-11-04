---
type: TestCase
id: TC-TRS-LSPTOK-001
name: "Semantic tokens colour variability references and flag unresolved ones"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/lsp_ple_tokens.rs
verifies:
  - REQ-TRS-LSPTOK-001
tags:
  - lsp
---

```gherkin
Feature: semantic tokens

  Scenario: legend and capability
    Then semanticTokens/full is advertised with the documented legend

  Scenario: references
    Then features, ids, parameter paths and stages are tokens in their fields, unknown ones carry the unresolved modifier, and operators, numbers, values and the body are not tokens

  Scenario: other files
    Then a file without these fields has no tokens
```
