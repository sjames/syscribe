---
id: TC-TRS-PARSE-011
type: TestCase
testLevel: L3
status: draft
name: "Verify an about: comment file attaches to each listed element, is not an element itself, and reports E027/W052."
verifies:
  - REQ-TRS-PARSE-011
---

Verify §3.10 `about:` comment files (GH #164).

```gherkin
Feature: about comment files

  Scenario: an about comment does not become an element
    Given SafetyNote.md with about VehicleSystem::Engine and VehicleSystem::Transmission
    When the tool validates the model and lists VehicleSystem
    Then no finding names SafetyNote.md
    And no element VehicleSystem::SafetyNote exists

  Scenario: the comment is shown on every listed element
    When the tool shows VehicleSystem::Engine and VehicleSystem::Transmission
    Then each prints the comment body under Note: SafetyNote
    And a requirement listed by its stable id shows the comment too

  Scenario: an unresolved entry raises E027
    Given a comment listing VehicleSystem::Engine and VehicleSystem::Ghost
    When the tool validates the model
    Then E027 names the comment file and VehicleSystem::Ghost
    And the comment still attaches to VehicleSystem::Engine
    Given a comment whose only entry is missing
    Then E027 is raised and the comment is kept as its own element

  Scenario: ignored fields raise W052
    Given a comment that also declares supertype
    And a package _index.md that declares about
    When the tool validates the model
    Then W052 names the comment's supertype field
    And W052 names the _index.md about field
```
