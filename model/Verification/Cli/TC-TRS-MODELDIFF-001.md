---
type: TestCase
id: TC-TRS-MODELDIFF-001
name: "changes lists element-level differences between revisions"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/model_changes_and_new_findings.rs
verifies:
  - REQ-TRS-MODELDIFF-001
tags:
  - cli
---

```gherkin
Feature: Element-level model diff

  Scenario: a commit flips two statuses and adds a test case
    Then exactly three entries are reported, the test case carrying its verifies link

  Scenario: a formatting-only frontmatter change
    Then no entry is reported

  Scenario: an element is moved
    Then one moved entry is reported against the working tree

  Scenario: a referenced element is removed
    Then the reference is listed as newly broken, and newly resolved in reverse

  Scenario: filters and errors
    Then --types narrows the result and an unknown revision exits 1
```
