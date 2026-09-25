---
id: TC-TRS-IMPL-003
type: TestCase
testLevel: L3
status: draft
name: "Verify package-registry implementedBy references are external (no W023) while missing local paths still raise W023."
verifies:
  - REQ-TRS-IMPL-003
---

```gherkin
Feature: package-registry implementedBy references (TC-TRS-IMPL-003)

  Scenario: registry references raise no W023
    Given a non-draft PartDef whose implementedBy lists crates.io, npm and github references
    When the tool validates the model
    Then no W023 names that PartDef

  Scenario: a missing local path still raises W023
    Given a non-draft PartDef whose implementedBy is a missing local path
    When the tool validates the model
    Then exactly one W023 is raised, naming that PartDef

  Scenario: sbom still maps the registry references to purls
    When the tool runs sbom
    Then the output contains pkg:cargo/tokio@1.38.0 and pkg:npm/lodash@4.17.21
```
