---
id: TC-TRS-QNAME-002
type: TestCase
testLevel: L3
status: draft
name: "Verify that the qualified-name package segment is the directory name, not the _index.md name: label."
verifies:
  - REQ-TRS-QNAME-002
---

Verify that the qualified-name package segment is the directory name, not the `_index.md` `name:` label.

```gherkin
Feature: Package qualified-name segment comes from the directory

  Scenario: name: in _index.md does not replace the directory name
    Given a directory Pkg/ with _index.md containing name: VS
    And a file Pkg/Engine.md with type: PartDef
    When the tool is invoked
    Then Engine.md has qualified name Pkg::Engine
    And no qualified name VS::Engine is reported
```
