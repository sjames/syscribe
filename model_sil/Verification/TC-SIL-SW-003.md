---
type: TestCase
id: TC-SIL-SW-003
title: Inspection — Independent development process evidence for Channel A and B diversity
status: active
testLevel: L2
verifies:
  - REQ-SIL-SW-002
---

```gherkin
Feature: Vital software diversity is achieved by independent development teams

  Scenario: Channel A and Channel B development records show no shared artefacts
    Given the Development Independence Evidence Package (DIEP) compiled by the ISA
    When the DIEP is reviewed for shared artefacts between the Channel A and Channel B development teams
    Then the following shared artefacts shall be absent:
      | Artefact type             | Required status           |
      | Personnel roster          | No personnel in common    |
      | Development tools         | Different compiler toolchains (e.g., GHS MULTI for Channel A, IAR Embedded Workbench for Channel B) |
      | RTOS configuration        | Different RTOS configurations (VxWorks vs. ThreadX or equivalent) |
      | Intermediate code artefacts | No shared object files, libraries, or generated code between teams |
      | Source code review records  | Reviews conducted independently; no shared review comments |
    And the ISA sign-off statement shall confirm "no common-cause software failure mode exists between Channel A and Channel B implementations"

  Scenario: Both channels independently translate the same B-Method specification
    Given the formally-proved B-Method abstract machine corpus (REQ-SIL-SW-003)
    When the Channel A development team and Channel B development team independently translate the B-Method abstract machine invariants into implementation code
    Then each team shall produce independent code artefacts derived solely from the formal specification
    And no code review, design discussion, or shared design document shall cross the team boundary after the B-Method specification is frozen
    And both implementations shall pass the same functional test suite (TC-SIL-SAFE-001 through TC-SIL-SAFE-004) independently before integration

  Scenario: Compilation tool qualification evidence is independent
    Given the EN 50128 Table A.12 tool qualification obligation for T3 tools
    When the tool qualification report for the Channel A compiler is reviewed alongside the Channel B compiler report
    Then each report shall cover a distinct compiler product
    And each report shall be assessed by a different independent tool qualification assessor
    And both tool qualification reports shall be accepted by the ISA before the compiled code is used in acceptance testing
```
