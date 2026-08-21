---
type: TestCase
id: TC-TOY-001
name: "Pressure sensor reads within range"
testLevel: L1
status: draft
verifies:
  - Legacy::PressureSensor
---

Verifies `Legacy::PressureSensor` — a `PartDef` plugin-synthesized by
`../Legacy/widgets.toy`, not a native `Requirement`. Legal only because it
was actually synthesized by a stdio plugin (`ADR-SYS-PLUGIN-002`'s widening
of `E104`'s `verifies:` target-legality check) — an ordinary hand-authored
`PartDef` outside a `foreignFormat:` package would still be rejected.

```gherkin
Feature: Pressure sensing
  Scenario: reading stays within range
    Given the pressure sensor is powered
    When it takes a reading
    Then the reading is within its rated range
```
