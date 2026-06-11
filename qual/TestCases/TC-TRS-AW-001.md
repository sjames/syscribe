---
id: TC-TRS-AW-001
type: TestCase
testLevel: L3
status: draft
title: "Verify the applies-when CLI: set by feature id or path, E209/E228 refusal, void-model bad-config check on set, clear, dry-run."
verifies:
  - REQ-TRS-AW-001
---

```gherkin
Feature: authoring appliesWhen from the CLI with a feature-model safety check
  Scenario: set a gate by feature id
    Given a sound feature model and a Requirement
    When applies-when --set is given a FEAT-* id
    Then the appliesWhen field is written and the command exits zero

  Scenario: set a gate by feature path (qualified name)
    Given a sound feature model and a Requirement
    When applies-when --set is given a FeatureDef qualified name
    Then the appliesWhen field is written and the command exits zero

  Scenario: an unresolved feature operand is refused
    When applies-when --set references a feature that resolves to nothing
    Then E209 is raised, the file is unchanged, and the command exits non-zero

  Scenario: a forbidden target is refused
    When applies-when --set targets a FeatureDef
    Then E228 is raised, the file is unchanged, and the command exits non-zero

  Scenario: a void feature model is caught on set
    Given a void feature model (no valid configuration)
    When applies-when --set succeeds in writing the gate
    Then the feature-model bad-configuration check reports E223 and exits non-zero

  Scenario: clear removes the gate
    When applies-when --clear is run on a gated element
    Then the appliesWhen field is removed

  Scenario: dry-run writes nothing
    When applies-when --set is run with --dry-run
    Then the file is not modified
```
