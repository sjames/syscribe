---
type: TestCase
id: TC-GTI-SAF-005
name: "String protection and surge protection"
status: active
testLevel: L2
verifies:
  - REQ-GTI-SYS-034
  - REQ-GTI-SYS-035
---

```gherkin
Feature: Inspection and test of DC and AC protection

  Scenario: fuse and diode rating
    Given the DC input assembly
    When the fuse and diode ratings are inspected
    Then they are rated for 1000 V DC and for reverse current

  Scenario: surge protection
    Given the DC and AC terminals
    When the installed devices are inspected
    Then type-II devices are fitted on both sides
```
