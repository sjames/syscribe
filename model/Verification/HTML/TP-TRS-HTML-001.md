---
type: TestPlan
id: TP-TRS-HTML-001
name: "export-html — integration test plan"
status: draft
scope: integration
demonstrates: [REQ-TRS-HTML-000]
testCases:
  - TC-TRS-HTML-001
  - TC-TRS-HTML-002
  - TC-TRS-HTML-003
  - TC-TRS-HTML-004
  - TC-TRS-HTML-005
  - TC-TRS-HTML-006
  - TC-TRS-HTML-007
tags:
  - html-export
  - integration
---

Integration plan for the `export-html` subcommand: runs the command against a fixture model and
inspects the generated static site (files present, element/report page content, cross-links,
embedded diagrams, search index, custom CSS, and the no-network offline guarantee). Executed as
the `cargo test -p syscribe --test export_html` suite.
