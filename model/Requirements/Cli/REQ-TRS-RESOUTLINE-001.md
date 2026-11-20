---
type: Requirement
id: REQ-TRS-RESOUTLINE-001
name: "The rows of a Scenario Outline recorded through records give the outline its verdict"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - results
---

A result record whose `scenario` is `<outline title> [row N]` shall contribute to the verdict of the TestCase's Scenario Outline of that title, so that `trace`, `matrix`, `testplan` and `audit` see it as they see the rows of a Cucumber report or of `simulate --record` (GH #500). Rows are numbered `1..N` across the outline's `Examples:` tables. A failing row, or an explicit `fail` record of the outline title, shall fail the outline; it shall pass only when every one of its `N` rows has a pass (a waived row makes it waived, a flaky, not-run, blocked, inconclusive, ignored or missing row leaves it unknown, never a pass); a row number the table does not have shall count for nothing; an outline with no row recorded shall keep the verdict of its own record. The roll-up shall apply per environment when a TestCase or plan names several.
