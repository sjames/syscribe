---
type: TestCase
id: TC-DRONE-ROTOR-001
name: "Rotor assembly structural inspection"
status: active
testLevel: L2
verifies:
  - PropulsionSubsystem::Propulsion::RotorAssembly
tags:
  - propulsion
  - structural
---

Post-build inspection of the SysML v2-authored `RotorAssembly` part def: confirms
the built hardware matches the fuel port, fuel item, and thrust-rating attribute
declared on that definition. Demonstrates `REQ-TRS-SYSMLV2-004` — a native
`TestCase.verifies:` targeting a SysMLv2-originated element (here, by its
Syscribe-derived qualified name) exactly as it would a native `Requirement`.

```gherkin
Feature: Rotor assembly structural inspection

  Scenario: Built hardware matches the SysML v2 definition
    Given a completed rotor assembly build
    When the inspector checks the fuel port, fuel item, and thrust-rating attribute
    Then all three shall match the RotorAssembly part def exactly
```
