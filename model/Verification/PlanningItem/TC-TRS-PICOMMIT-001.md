---
type: TestCase
id: TC-TRS-PICOMMIT-001
name: "Commit evidence is checked against git and shown"
status: active
testLevel: L2
sourceFile: repo:crates/syscribe/tests/planning_commit_evidence.rs
verifies:
  - REQ-TRS-PICOMMIT-001
tags:
  - planning
---

```gherkin
Feature: Commits as PlanningItem evidence

  Scenario: resolving commit
    Then a done leaf whose only evidence is a resolving abbreviated commit raises neither E728 nor E719

  Scenario: malformed and unknown commits
    Then E728 names a non-hex hash, a too-short hash and an unknown hash, and the done leaf also raises E719

  Scenario: waived commit
    Then a commit entry with a rationale is not checked

  Scenario: no repository
    Then outside a git repository a well-formed commit raises neither E728 nor E719

  Scenario: commit paths
    Then with check_commit_paths W320 is raised for a commit touching none of the item's paths, not for one touching a ref's sourceFile, and not for an item naming no path; off by default

  Scenario: commit required
    Then with require_commit_evidence W321 is raised on a done leaf without commit evidence only; off by default

  Scenario: read views
    Then steps --json and context show the full sha, subject and status of the commit

  Scenario: set evidence.add commit
    Then a resolving commit is appended and an unknown or malformed one is refused with nothing written

  Scenario: log by item
    Then log --item lists the commits the item names, and a non-PlanningItem exits 1
```
