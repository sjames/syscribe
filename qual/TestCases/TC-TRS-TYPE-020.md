---
id: TC-TRS-TYPE-020
type: TestCase
testLevel: L3
status: draft
name: "Verify IEC 62443 Zone/Conduit: E950/E951/E954/E956 structural rules, W950 SL gap, W953 isolated zone; a clean model is silent; the zones / conduits / zones --coverage commands and template Zone/Conduit."
verifies:
  - REQ-TRS-TYPE-020
---

Verify Zone/Conduit recognition, validation, and the read-only CLI surface.

```gherkin
Feature: IEC 62443 Zone/Conduit (§13)

  Scenario: well-formed zones and conduit are clean
    Given a model with two zones, a conduit, and an in-zone part
    When the tool validates the model
    Then none of E950–E956 or W950 are emitted

  Scenario: E950 — zone missing targetSL
    Given a Zone with no targetSL
    When the tool validates the model
    Then an E950 finding is emitted

  Scenario: E951 — bad zone id
    Given a Zone whose id is not a ZN-* id
    When the tool validates the model
    Then an E951 finding is emitted

  Scenario: E954 — conduit endpoint not a zone
    Given a Conduit whose fromZone resolves to a non-Zone
    When the tool validates the model
    Then an E954 finding is emitted

  Scenario: E956 — part inZone not a zone
    Given a Part whose inZone resolves to a non-Zone
    When the tool validates the model
    Then an E956 finding is emitted

  Scenario: W950 — SL gap
    Given a Zone whose achievedSL is below targetSL
    When the tool validates the model
    Then a W950 finding is emitted

  Scenario: W953 — approved zone with no conduit
    Given an approved Zone (targetSL >= 2) referenced by no conduit
    When the tool validates the model
    Then a W953 finding is emitted

  Scenario: zones and conduits commands
    Given the clean model
    When `zones`, `conduits`, and `zones --coverage` are run
    Then they report the zones, the conduit, and the security control

  Scenario: template Zone and Conduit
    When the Zone and Conduit templates are printed
    Then they contain `type: Zone` and `type: Conduit`
```
