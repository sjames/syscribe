---
id: TC-TRS-MOVE-002
type: TestCase
testLevel: L3
status: draft
title: "Verify move updates all qualified-name references, including nested ones, without false matches."
verifies:
  - REQ-TRS-MOVE-002
---

Verify that moving an element rewrites every qualified-name reference to it — top-level (`supertype`) and nested (connection `from:`/`to:`, feature `typedBy:`) — leaves a prefix-sharing sibling untouched, and introduces no unresolved-reference findings.

```gherkin
Feature: Move updates all references

  Scenario: Top-level and nested references are rewritten
    Given elements referencing Pkg::Sub::Widget via supertype, a connection endpoint, and feature typedBy
    When move Pkg::Sub::Widget Pkg::Other::Widget is run
    Then each of those references now reads Pkg::Other::Widget

  Scenario: Companion SVG references are rewritten
    Given a companion .svg citing Pkg::Sub::Widget via sysml:ref/data-qname/href
    When move Pkg::Sub::Widget Pkg::Other::Widget is run
    Then those SVG attributes now read Pkg::Other::Widget while the sibling is untouched

  Scenario: Descendant references follow a package move
    Given a connection endpoint Pkg::Sub::Widget::port
    When move Pkg::Sub Pkg::Moved is run
    Then the endpoint now reads Pkg::Moved::Widget::port

  Scenario: Multi-segment references in the Markdown body are rewritten
    Given an ADR body citing `Pkg::Sub::Widget`
    When move Pkg::Sub::Widget Pkg::Other::Widget is run
    Then the ADR body now cites Pkg::Other::Widget

  Scenario: Prefix-sharing siblings are not affected
    Given an element Pkg::Sub::WidgetExtended referenced elsewhere in frontmatter and body
    When move Pkg::Sub::Widget Pkg::Other::Widget is run
    Then references to Pkg::Sub::WidgetExtended are unchanged

  Scenario: No dangling references remain
    Given a model that validates cleanly before the move
    When the move completes
    Then validation introduces no new unresolved-reference (E102/E103-class) findings
```
