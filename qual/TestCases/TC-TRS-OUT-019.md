---
id: TC-TRS-OUT-019
type: TestCase
testLevel: L3
status: draft
name: "Verify the sbom command: local paths → file components with model externalReferences; registry URIs → package components with PURLs; CycloneDX 1.6 and SPDX 2.3 are well-formed; --include-tests adds test components; --output writes a file."
verifies:
  - REQ-TRS-OUT-019
---

Verify SBOM generation from `implementedBy:` links against a part that mixes a local path
with crates.io / npm / github package URIs.

```gherkin
Feature: SBOM generation (§18)

  Scenario: CycloneDX is well-formed with file + package components
    Given a part implementedBy a local path and registry URIs
    When `sbom` is run
    Then the output is CycloneDX 1.6 with a tokio cargo PURL and a local scheduler component

  Scenario: local component links back to the satisfied requirement
    When `sbom` is run
    Then the scheduler component has a model externalReference to REQ-SBOM-001

  Scenario: registry URIs become PURLs for each ecosystem
    When `sbom` is run
    Then npm and github PURLs are present

  Scenario: SPDX 2.3 output
    When `sbom --format spdx` is run
    Then the output is SPDX-2.3 with GENERATED_FROM relationships

  Scenario: --include-tests adds test source components
    When `sbom --include-tests` is run
    Then the test source file is a component

  Scenario: --output writes a file
    When `sbom --output <file>` is run
    Then the file contains the BOM
```
