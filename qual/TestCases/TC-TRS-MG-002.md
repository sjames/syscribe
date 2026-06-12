---
id: TC-TRS-MG-002
type: TestCase
testLevel: L3
status: draft
name: "Verify gated actor validation: inert without the gate; MG010 unresolved, MG011 non-part, MG012 not external, MG013 no actors; actorIn index."
verifies:
  - REQ-TRS-MG-002
---

```gherkin
Feature: MagicGrid gate validates use-case actors
  Scenario: actors are inert without the gate
    Given a UseCaseDef with a dangling actors entry
    When the model is validated without the magicgrid profile
    Then no actor-related finding is produced

  Scenario: an unresolved actor raises MG010 under the gate
    Given a UseCaseDef whose actors names no model element
    When validate --profile magicgrid is run
    Then MG010 is raised against the use case

  Scenario: an actor that is not a part raises MG011
    Given a UseCaseDef whose actors names a Requirement
    When validate --profile magicgrid is run
    Then MG011 is raised naming the resolved type

  Scenario: a non-external actor raises MG012
    Given a UseCaseDef whose actor part is not marked mg_external true
    When validate --profile magicgrid is run
    Then MG012 is raised against the use case

  Scenario: a use case with no actors raises MG013
    Given a non-draft UseCaseDef with an empty actors list
    When validate --profile magicgrid is run
    Then MG013 is raised

  Scenario: actor participation is indexed
    Given an external actor part referenced by two use cases
    When the model is processed under the gate
    Then the actor reports both use cases under actorIn
```
