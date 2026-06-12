---
id: TC-TRS-SAFE-009
type: TestCase
testLevel: L3
status: draft
name: "Verify W039 fires for silLevel 3 and 4 SafetyGoals missing I3 assessment"
verifies:
  - REQ-TRS-SAFE-009
---

## Scenario: silLevel 4 goal without I3 assessment raises W039

**Given** a model with a SafetyGoal carrying `silLevel: 4` and at least one
ConfirmationMeasure present (dormant-trigger satisfied), but no I3
functional_safety_assessment confirming the goal
**When** the tool validates
**Then** W039 is raised naming the SafetyGoal

## Scenario: silLevel 3 goal without I3 assessment raises W039

**Given** a model with a SafetyGoal carrying `silLevel: 3` and the same ConfirmationMeasure
**When** the tool validates
**Then** W039 is raised naming the SafetyGoal

## Scenario: silLevel 2 goal does not raise W039

**Given** a model with a SafetyGoal carrying `silLevel: 2`
**When** the tool validates
**Then** W039 is not raised for this goal

## Scenario: asilLevel D still triggers W039 (regression)

**Given** a model with a SafetyGoal carrying `asilLevel: D`, same conditions
**When** the tool validates
**Then** W039 is raised naming the SafetyGoal

## Scenario: no ConfirmationMeasure — opt-in dormant

**Given** a model with only a silLevel 4 SafetyGoal and no ConfirmationMeasure at all
**When** the tool validates
**Then** W039 is not raised (opt-in invariant)
