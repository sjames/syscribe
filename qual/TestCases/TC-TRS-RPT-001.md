---
id: TC-TRS-RPT-001
type: TestCase
testLevel: L3
status: draft
name: "Verify fmea report emits RPN-sorted Markdown table and fault-tree render emits Mermaid flowchart"
verifies:
  - REQ-TRS-RPT-001
---

```gherkin
Feature: fmea report and fault-tree render commands

  Scenario: fmea report prints Markdown table sorted by RPN descending
    Given a model with two FMEASheet entries: one with RPN 729 and one with RPN 56
    When the user runs fmea report
    Then the output is a Markdown table with columns ID, Name, Failure Mode, Effect, Severity, Occurrence, Detection, RPN, Controls, Status
    And the row with RPN 729 appears before the row with RPN 56

  Scenario: fmea report --json emits a JSON array
    Given the same FMEA model
    When the user runs fmea report --json
    Then the output is a JSON array where each entry has an rpn field

  Scenario: fmea report --fmea-sheet filters to named sheet
    Given a model with two FMEASheet elements FM-KERN and FM-OTHER
    When the user runs fmea report --fmea-sheet FM-KERN
    Then only entries from FM-KERN appear in the output

  Scenario: fault-tree render emits Mermaid flowchart
    Given a model with a FaultTree element FT-KERN-001 containing at least two events and a gate
    When the user runs fault-tree render FT-KERN-001
    Then the output starts with "flowchart TD"
    And the output contains node ids for each FaultTreeEvent in the tree

  Scenario: fault-tree render of unknown id exits with error
    Given no FaultTree element named FT-NONEXIST-001 in the model
    When the user runs fault-tree render FT-NONEXIST-001
    Then the tool exits with a non-zero exit code and an informative error message
```
