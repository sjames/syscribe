---
type: TestCase
id: TC-TRS-REQGRAPH-001
name: "req-graph returns bounded typed neighbourhoods with overlays and an overview"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe-server/tests/req_graph.rs
verifies:
  - REQ-TRS-REQGRAPH-001
tags:
  - server
---

```gherkin
Feature: requirement graph API

  Scenario: neighbourhood
    Then depth, edge filters, the node cap and the overlays shape the returned subgraph

  Scenario: configuration and errors
    Then a configuration removes gated elements and bad input is 400/404

  Scenario: overview
    Then counts by class and status and the unlinked requirements are reported
```
