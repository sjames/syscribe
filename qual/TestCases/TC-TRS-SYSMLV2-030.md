---
id: TC-TRS-SYSMLV2-030
type: TestCase
testLevel: L3
status: active
name: "Verify an ingested SysMLv2 file with parsed-but-unmapped constructs raises exactly one advisory W543 listing per-kind counts, and a fully mapped file raises none."
verifies:
  - REQ-TRS-SYSMLV2-030
sourceFile: repo:crates/syscribe-model/tests/sysmlv2_unmapped.rs
testFunctions:
  - one_w543_per_file_with_per_kind_counts
  - fully_mapped_file_is_silent
  - nested_package_members_are_counted
---

```gherkin
Feature: unmapped SysMLv2 constructs are reported (TC-TRS-SYSMLV2-030)

  Scenario: one W543 per file with per-kind counts
    Given a sysmlSubmodel file declaring two metadata defs, an occurrence def and an alias
    When the tool validates the model
    Then exactly one W543 is raised for that file and it lists metadata def x2, occurrence def x1 and alias x1

  Scenario: a fully mapped file is silent
    Given a sysmlSubmodel file declaring only a part def
    When the tool validates the model
    Then no W543 is raised

  Scenario: the finding is advisory
    Given a file with unmapped constructs and a mapped part def
    When the tool validates the model
    Then W543 is a warning and the part def is still synthesized
```
