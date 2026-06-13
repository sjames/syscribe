---
id: TC-TRS-OUT-020
type: TestCase
testLevel: L3
status: draft
name: "Verify export-reqif: well-formed ReqIF 1.2 XML; a SPEC-OBJECT per Requirement; nested SPEC-HIERARCHY for packages; DERIVED_FROM relations; --include-tests adds VERIFIED_BY; --zip writes a readable .reqifz."
verifies:
  - REQ-TRS-OUT-020
---

Verify ReqIF export against a small two-package requirement tree with a derivedFrom link.

```gherkin
Feature: ReqIF 1.2 export (§21)

  Scenario: output is well-formed ReqIF XML
    Given a model with requirements
    When `export-reqif` is run
    Then the output is well-formed XML

  Scenario: a SPEC-OBJECT per requirement
    When `export-reqif` is run
    Then each requirement id appears as a SPEC-OBJECT

  Scenario: package hierarchy as nested SPEC-HIERARCHY
    When `export-reqif` is run
    Then SPEC-HIERARCHY entries are present

  Scenario: derivedFrom becomes a DERIVED_FROM relation
    When `export-reqif` is run
    Then a SPEC-RELATION of type DERIVED_FROM is present

  Scenario: --include-tests adds VERIFIED_BY
    When `export-reqif --include-tests` is run
    Then a VERIFIED_BY relation is present

  Scenario: --zip writes a readable .reqifz
    When `export-reqif --zip --output <file>` is run
    Then the .reqifz archive contains content.reqif
```
