---
id: TC-TRS-MG-003
type: TestCase
testLevel: L3
status: draft
title: "Verify mg_cell classification and the magicgrid grid report: MG020 invalid coord, MG021 type/column mismatch, grid render with empty-cell flag, --json."
verifies:
  - REQ-TRS-MG-003
---

```gherkin
Feature: MagicGrid cell classification and grid report
  Scenario: a classified element appears in its grid cell
    Given a UseCaseDef with mg_cell B2
    When the magicgrid report is rendered
    Then the element is listed in the Black-box Behavior cell

  Scenario: an invalid coordinate raises MG020 under the gate
    Given an element with mg_cell X9
    When validate --profile magicgrid is run
    Then MG020 is raised

  Scenario: a type/column mismatch raises MG021 under the gate
    Given a PartDef with mg_cell B1
    When validate --profile magicgrid is run
    Then MG021 is raised naming the type/column conflict

  Scenario: the grid report flags empty cells
    Given a model populating some but not all cells
    When the magicgrid report is rendered
    Then the full B/W/S by 1-4 grid is printed with per-cell counts and empty cells are marked

  Scenario: the grid report emits JSON
    When magicgrid --json is run
    Then a JSON grid structure is produced

  Scenario: mg_cell is inert without the gate
    Given an element with mg_cell X9
    When the model is validated without the magicgrid profile
    Then no MG02x finding is produced
```
