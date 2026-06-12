---
id: REQ-TRS-TAG-002
type: Requirement
name: "list command shall support multi-tag AND filtering to narrow test selection"
status: draft
reqDomain: software
verificationMethod: test
derivedFrom:
  - REQ-TRS-TAG-001
---

The `list` command's `--tag` flag **shall** accept multiple values by repeating the flag
(e.g. `--tag integration --tag safety`). When more than one `--tag` is given, the tool
**shall** return only elements that carry **all** of the specified tags (AND semantics),
narrowing rather than broadening the result set.

This differs from the OR semantics in [[REQ-TRS-TAG-001]], which applies to other tag-aware
commands. The distinction exists because `list` is the primary test-selection surface for CI
runners, where the goal is to narrow to a specific workload (e.g. "all integration tests that
are also safety-tagged"), not to union multiple groups.

**Acceptance criteria:**

- `list TestCase --tag a` returns every TestCase carrying tag `a`.
- `list TestCase --tag a --tag b` returns only TestCases carrying **both** `a` and `b`; an
  element carrying only `a` or only `b` is excluded.
- `list TestCase` with no `--tag` returns all TestCases (filter is inactive when no tag given).
- Single-tag behaviour is unchanged from REQ-TRS-TAG-001 for the `list` command.
