---
id: TC-TRS-MOVE-001
type: TestCase
testLevel: L3
status: draft
name: "Verify move relocates an element and a package (with subtree) and rejects invalid destinations."
verifies:
  - REQ-TRS-MOVE-001
---

Verify that `move` relocates a single element file and an entire package subtree to the path derived from the destination qualified name, and rejects moves onto an existing target, onto itself, or into its own subtree.

```gherkin
Feature: Move element or package

  Scenario: Move a single element to a new namespace
    Given an element at qualified name Pkg::Sub::Widget
    When move Pkg::Sub::Widget Pkg::Other::Widget is run
    Then the file for Pkg::Other::Widget exists and the old path is gone

  Scenario: Move a package relocates its whole subtree
    Given a package Pkg::Sub containing child elements
    When move Pkg::Sub Pkg::Moved is run
    Then every child is reachable under Pkg::Moved and none under Pkg::Sub

  Scenario: Moving onto an existing destination is rejected
    Given two elements Pkg::A and Pkg::B
    When move Pkg::A Pkg::B is run
    Then the command exits non-zero and nothing changes

  Scenario: Moving a package into its own subtree is rejected
    Given a package Pkg::Sub
    When move Pkg::Sub Pkg::Sub::Inner is run
    Then the command exits non-zero and nothing changes
```
