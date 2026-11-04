---
type: TestCase
id: TC-TRS-TIMELINE-001
name: "timeline lists each stage's decisions, the effective provenance and what is open"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/bind_timeline.rs
verifies:
  - REQ-TRS-TIMELINE-001
tags:
  - cli
---

```gherkin
Feature: stage timeline

  Scenario: the chain
    Then the chain is root first, each step lists only its own decisions and stage, and effective values carry source and stage

  Scenario: open items and surfaces
    Then what is still open is listed by stage and the CLI, MCP tool and LSP request agree
```
