---
type: TestCase
id: TC-TRS-TRACECODE-001
name: "trace-code finds untraced units by the unit rules and the containment rule"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/trace_code.rs
verifies:
  - REQ-TRS-TRACECODE-001
tags:
  - cli
---

```gherkin
Feature: trace-code lists code no element names

  Scenario: unit rules and containment
    Given cargo crates, build directories and files under the [traceCode] roots
    When trace-code runs
    Then a unit named by a file inside it, a directory above it, itself or a sourceFile is traced
    And excluded trees, target/ and files with other extensions are not units
    And the rest are listed as untraced and the command exits 2

  Scenario: configuration errors
    Then no [traceCode] table, a malformed entry, an unreadable baseline or a bad flag exits 1

  Scenario: MCP
    Then the trace_code tool returns the same report

  Scenario: discoverable
    Then help --search finds trace-code for untraced, code without design, implementedBy and trace code
    And explain describes W326, W327 and W328
```
