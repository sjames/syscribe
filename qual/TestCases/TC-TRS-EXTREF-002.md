---
id: TC-TRS-EXTREF-002
type: TestCase
testLevel: L3
status: draft
name: "Verify extref lookup command, --json, show surfacing, and spec fields listing."
verifies:
  - REQ-TRS-EXTREF-002
---

Verify that `extref <ref>` looks up elements by external reference (exact match, all matches on a duplicate, non-zero on a miss), that `--json` emits an array, that `show` surfaces the field, and that `spec fields` lists it.

```gherkin
Feature: extRef lookup and discoverability

  Scenario: extref finds the element declaring a reference
    Given a model where one element declares extRef "DNG:4521"
    When the tool runs `extref DNG:4521`
    Then the matching element's qualified name is printed
    And the tool exits zero

  Scenario: extref returns all elements sharing a duplicate reference
    Given two elements that both declare extRef "DUP:1"
    When the tool runs `extref DUP:1`
    Then both elements are printed

  Scenario: extref on an unknown reference reports no match and exits non-zero
    Given a model with no element declaring "NOPE:0"
    When the tool runs `extref NOPE:0`
    Then no element is printed
    And the tool exits non-zero

  Scenario: extref --json emits an array of matches
    Given a model where one element declares extRef "DNG:4521"
    When the tool runs `extref DNG:4521 --json`
    Then the output is a JSON array containing the element

  Scenario: show surfaces extRef on an element that declares it
    Given an element that declares extRef
    When the tool runs `show` on that element
    Then the external reference is displayed

  Scenario: spec fields lists extRef
    When the tool runs `spec fields`
    Then extRef appears in the field reference
```
