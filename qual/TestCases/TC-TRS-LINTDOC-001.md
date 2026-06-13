---
id: TC-TRS-LINTDOC-001
type: TestCase
testLevel: L3
status: draft
name: "Verify lint-docs diagram resolution: W100 for an unresolved Mermaid qualified name, W101 for a stale SVG sysml:ref, W102 for a missing image embed; resolving refs and prose qnames are clean; --json shape."
verifies:
  - REQ-TRS-LINT-002
---

Verify the `lint-docs` diagram-reference checks against a small model + docs fixture.

```gherkin
Feature: lint-docs diagram references (§ GH #74)

  Scenario: W100 — unresolved Mermaid qualified name
    Given a doc with a ```mermaid block referencing Ghost::Element
    When `lint-docs` scans it against the model
    Then a W100 finding is emitted

  Scenario: W102 — missing image embed
    Given a doc embedding ![](missing.svg)
    When `lint-docs` scans it
    Then a W102 finding is emitted

  Scenario: W101 — stale SVG sysml:ref
    Given an SVG with sysml:ref="Gone::Element"
    When `lint-docs` scans it
    Then a W101 finding is emitted

  Scenario: resolving refs and prose qnames are clean
    Given a doc whose Mermaid references the existing Engine and whose prose mentions a qname
    When `lint-docs` scans it
    Then no diagram findings are emitted

  Scenario: --json shape
    When `lint-docs --json` reports a finding
    Then it includes file, line, code, and ref
```
