---
type: TestCase
id: TC-TRS-HEALTH-001
name: "health counts the right things, stays cheap and is cached per model version"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/health.rs
verifies:
  - REQ-TRS-HEALTH-001
tags:
  - cli
---

```gherkin
Feature: project health

  Scenario: counts and click-through
    Then errors, orphan and unverified requirements and failing Configurations are counted, each non-zero metric names its command, and fixing the problems clears them

  Scenario: budget and caching
    Then the largest example is summarised within the budget, and the MCP and LSP answers are cached until the model changes
```
