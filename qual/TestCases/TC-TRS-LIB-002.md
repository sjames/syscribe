---
id: TC-TRS-LIB-002
type: TestCase
testLevel: L3
status: draft
title: "Verify SI/ISQ recognition (open/curated tier): recognised members resolve clean (no W404), unknown members lenient (no W043), unit: permissive; closed-package W043 unaffected."
verifies:
  - REQ-TRS-LIB-002
---

```gherkin
Feature: recognise SI units and ISQ quantity-value types as a lenient curated library
  Scenario: recognised ISQ type resolves cleanly
    Given an operation parameter typed by ISQ::MassValue
    When validate runs
    Then no W404 is raised for it

  Scenario: an unknown ISQ/SI member is lenient
    Given a typedBy of ISQ::WibbleValue and a feature using SI units
    When validate runs
    Then no W043 is raised for any ISQ/SI member (open tier is lenient)

  Scenario: the closed-package typo check is unaffected
    Given a typedBy of ScalarValues::Flota
    When validate runs
    Then W043 is still raised for it

  Scenario: domain units stay permissive
    Given features with unit: USD / kWh / percent
    When validate runs
    Then none of them is flagged
```
