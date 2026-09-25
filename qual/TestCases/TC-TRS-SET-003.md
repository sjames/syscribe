---
id: TC-TRS-SET-003
type: TestCase
testLevel: L3
status: draft
name: "Verify set evidence.add is a no-op for an existing entry and the list operations preserve YAML comments."
verifies:
  - REQ-TRS-SET-003
---

Verify that `syscribe set <PI> evidence.add` does not append a duplicate of an
existing entry, and that `evidence.add`/`achieves.add` insert only the new
item's lines, leaving comments and every other frontmatter line untouched
(GH #152).

```gherkin
Feature: set list operations are idempotent and surgical

  Scenario: evidence.add of an existing ref is a reported no-op
    Given PlanningItem PI-SET-003 whose evidence: already has ref REQ-SET-011
    When the tool runs set PI-SET-003 evidence.add ref=REQ-SET-011
    Then it exits 0, reports the entry is already present and leaves the file byte-identical

  Scenario: evidence.add of an existing path is a reported no-op
    Given PI-SET-003 whose evidence: already has path src/real.rs
    When the tool runs set PI-SET-003 evidence.add path=src/real.rs
    Then it exits 0, reports the entry is already present and leaves the file byte-identical

  Scenario: evidence.add of a new entry preserves YAML comments and every other line
    Given PI-SET-003 whose frontmatter carries full-line and trailing YAML comments
    When the tool runs set PI-SET-003 evidence.add ref=REQ-SET-010 rationale=Covered by review
    Then every original line, comments included, is unchanged
    And exactly the two lines of the new entry are added at the list's indentation
    And the model still validates

  Scenario: achieves.add of a new requirement preserves YAML comments
    Given the same PI-SET-003
    When the tool runs set PI-SET-003 achieves.add REQ-SET-011
    Then every original line is unchanged and REQ-SET-011 is appended to achieves:
```
