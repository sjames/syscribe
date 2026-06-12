---
id: TC-TRS-QNAME-004
type: TestCase
testLevel: L3
status: draft
name: "Verify that _index.md contributes no name segment to its package or sibling elements."
verifies:
  - REQ-TRS-QNAME-004
---

Verify that _index.md contributes no name segment to its package or sibling elements.

```gherkin
Feature: _index.md contributes no name segment

  Scenario: _index.md does not add _index to any qualified name
    Given a directory Pkg/ containing _index.md and Foo.md
    When the tool is invoked
    Then Foo.md has qualified name Pkg::Foo
    And no element has a qualified name containing the segment _index

  Scenario: _index.md's own qualified name is the package name only
    Given a directory Pkg/ containing _index.md with type: Package and name: Pkg
    When the tool is invoked
    Then _index.md's element has qualified name Pkg, not Pkg::_index
```
