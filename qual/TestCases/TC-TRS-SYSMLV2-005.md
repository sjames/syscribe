---
id: TC-TRS-SYSMLV2-005
type: TestCase
testLevel: L3
status: draft
name: "Verify @SyscribeFeature lifts a SysMLv2 variant into appliesWhen, feature-check --deep/--config project it correctly, and a no-annotation variant stays purely structural."
verifies:
  - REQ-TRS-SYSMLV2-005
---

```gherkin
Feature: @SyscribeFeature targets a native FeatureDef
  Scenario: a variant carrying @SyscribeFeature is gated like a native appliesWhen element
    Given a SysMLv2 variant carrying @SyscribeFeature { featureId = '<FEAT-id>'; } and two Configurations selecting that feature true/false
    When --config is run against each Configuration
    Then the variant is present under the true selection and absent under the false selection

  Scenario: feature-check --deep reports the feature model as sound
    Given the same variant and Configurations
    When feature-check --deep is run
    Then it reports zero errors and a valid model for both Configurations

  Scenario: a variant with no annotation stays purely structural
    Given a SysMLv2 variant with no @SyscribeFeature annotation
    When --config is run against either Configuration
    Then the variant is present in both and raises no feature-model finding

  Scenario: an unresolvable featureId is a dangling-reference finding
    Given a SysMLv2 variant carrying @SyscribeFeature { featureId = '<unknown-id>'; }
    When the model is validated
    Then the existing unresolved-feature-reference finding is raised
```
