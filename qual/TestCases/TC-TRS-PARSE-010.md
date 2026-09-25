---
id: TC-TRS-PARSE-010
type: TestCase
testLevel: L3
status: draft
name: "Verify a locale variant file attaches its body to the base element (no new element, no W042), with E026/W051 for dangling, duplicate and structural variants."
verifies:
  - REQ-TRS-PARSE-010
---

Verify §3.10 locale documentation variants (GH #160).

```gherkin
Feature: locale documentation variants

  Scenario: a variant file does not become an element
    Given VehicleSystem::Engine and the variants Engine.de.md, Engine.fr.md and Engine.it.md
      each with qualifiedName VehicleSystem::Engine and a locale
    When the tool validates the model
    Then no W042 finding is raised
    And ls VehicleSystem lists VehicleSystem::Engine but no VehicleSystem::Engine.<suffix> element

  Scenario: the variant bodies are shown on the base element
    When the user runs show VehicleSystem::Engine
    Then the output has a "Documentation (de)" section with the German body
    And a "Documentation (fr)" section with the French body

  Scenario: a duplicate locale and a structural field raise W051
    Given Engine.zz_de_copy.md, a second de variant of VehicleSystem::Engine
      and Engine.it.md, a variant that also sets isAbstract
    When the tool validates the model
    Then a W051 finding names Engine.zz_de_copy.md
    And a W051 finding names Engine.it.md and the field isAbstract
    And the first de variant's body is the one shown

  Scenario: a variant naming a missing element raises E026
    Given Ghost_de.md with qualifiedName VehicleSystem::Ghost, which no element has
    When the tool validates the model
    Then an E026 finding names Ghost_de.md and VehicleSystem::Ghost
```
