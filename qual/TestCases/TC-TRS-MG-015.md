---
id: TC-TRS-MG-015
type: TestCase
testLevel: L3
status: draft
title: "Verify the magicgrid report renders the 3x4 B/W/S × pillar grid matrix (counts, SoI marker, empty cells) alongside the per-cell detail."
verifies:
  - REQ-TRS-MG-015
---

```gherkin
Feature: magicgrid renders the 3x4 grid matrix
  Scenario: the report contains a grid matrix
    Given a MagicGrid model with populated B/W/S cells and a B3 System of Interest
    When magicgrid runs
    Then a grid table with a Row header and the four pillar columns is shown
    And each row shows per-cell counts
    And the B3 System-of-Interest cell is marked
    And empty cells are marked
    And the populated-cell count is shown
    And the per-cell element detail is still present
```
