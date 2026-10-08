---
id: TC-TRS-MCP-MEM-000
type: TestCase
testLevel: L3
status: active
name: "Verify the MCP server reports its memory and keeps a 12,000-element model within budget without growth across guarded writes."
verifies:
  - REQ-TRS-MCP-MEM-000
sourceFile: repo:crates/syscribe/tests/mcp_memory.rs
testFunctions:
  - server_stats_reports_the_model_and_memory
  - an_element_record_stays_small
  - twelve_thousand_elements_stay_within_budget_and_do_not_grow_with_writes
tags:
  - mcp
  - memory
---

Run with `cargo test -p syscribe --test mcp_memory`. The large-model test reads memory from the
server's own `server_stats` tool and runs on Linux only.

```gherkin
Feature: MCP memory (TC-TRS-MCP-MEM-000)

  Scenario: observable
    When server_stats is called
    Then it reports elements, body bytes, record size and, on Linux, resident and peak memory

  Scenario: compact records
    Then one element record is under 3,000 bytes

  Scenario: a 12,000-element model
    When it is loaded and three guarded writes are made
    Then resident memory after load is under 160 MB, the peak under 190 MB, and later writes do not add more than 25 MB
```
