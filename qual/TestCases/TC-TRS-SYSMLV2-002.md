---
id: TC-TRS-SYSMLV2-002
type: TestCase
testLevel: L3
status: draft
name: "Verify native SysMLv2 parsing + qname-mapped merge: multi-file package merge, qname derivation, and parse-failure isolation."
verifies:
  - REQ-TRS-SYSMLV2-002
---

```gherkin
Feature: native parsing and qname-mapped merge into the graph
  Scenario: two files contributing to the same SysML v2 package merge into one namespace
    Given two .sysml files each declaring part of the same package name
    When the model is validated
    Then elements from both files resolve under one merged qualified-name namespace

  Scenario: a nested SysML v2 package derives a full-depth qualified name
    Given a SysML v2 package nested inside another package
    When the model is validated
    Then the nested element's qualified name reflects the full nesting depth

  Scenario: a parse failure in one file does not abort the rest of the subtree
    Given one syntactically broken .sysml file alongside one well-formed .sysml file
    When the model is validated
    Then a warning names the broken file, the well-formed file's elements still appear, and validation completes with no crash
```
