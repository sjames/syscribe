---
id: TC-TRS-ID-006
type: TestCase
testLevel: L3
status: draft
title: "Verify FeatureDef stable FEAT id: mandatory id (E201), id-or-qname feature references in appliesWhen and Configuration features; E006/E101/E209 rules."
verifies:
  - REQ-TRS-ID-006
---

```gherkin
Feature: FeatureDef stable id and feature-by-id references (REQ-TRS-ID-006)
  Scenario: a feature referenced by its FEAT id gates identically to its qname
    Given a FeatureDef Anti_Lock with id FEAT-ABS-001
    And a Configuration selecting it via the FEAT-ABS-001 key
    And a Requirement with appliesWhen FEAT-ABS-001
    When validate runs
    Then no E209 is raised
    And the Requirement is active in the selecting configuration
    And the Requirement is inactive in a deselecting configuration

  Scenario: the FEAT-id form and the qname form are equivalent
    Given a parallel model that references the same feature by its qualified name
    When the configurations are projected
    Then the Requirement's activation matches the FEAT-id form exactly

  Scenario: a Configuration may key its features selection by the FEAT id
    Given a Configuration with features FEAT-ABS-001 true
    When the configuration is projected
    Then the feature is selected, gating the FEAT-id requirement active

  Scenario: hyphen relaxation applies only to the stable-id form
    Given a Requirement with appliesWhen Features::Anti-Lock (a hyphenated name)
    When validate runs
    Then E209 is raised (no regression of the basic-name rule)

  Scenario: a FEAT id without a trailing number is accepted
    Given a FeatureDef with id FEAT-ABS (no numeric suffix)
    When validate runs
    Then no E006, E023, or E201 is raised

  Scenario: a feature with no id is rejected (the FEAT id is mandatory)
    Given a FeatureDef declaring a name but no id
    When validate runs
    Then E201 is raised naming the FeatureDef as requiring an id

  Scenario: a malformed FEAT id is rejected
    Given a FeatureDef with id FEAT-bad
    When validate runs
    Then E006 is raised

  Scenario: two FeatureDefs sharing a FEAT id collide
    Given two FeatureDefs both declaring id FEAT-ABS-001
    When validate runs
    Then E101 is raised

  Scenario: a stable-id-shaped reference that resolves to nothing is unresolved
    Given a Requirement with appliesWhen FEAT-NOPE-001
    When validate runs
    Then E209 is raised
```
