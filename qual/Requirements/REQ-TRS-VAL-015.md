---
id: REQ-TRS-VAL-015
type: Requirement
name: Tool shall emit informational I010 for non-active TestCases whose sources are not yet present
status: draft
reqDomain: software
verificationMethod: test
---

So that a planned verification gap stays *visible* without being conflated with active drift, the tool **shall** emit an **informational** finding `I010` for a non-active `TestCase` (status `draft`, `review`, or `approved`) whose `sourceFile` does not resolve to an existing local file, or whose `testFunctions[].function` does not resolve in an existing `sourceFile`.

`I010` **shall** carry an *informational* severity, distinct from `Error` and `Warning`:

- it **shall not** by itself cause a non-zero exit (it is neither an error nor a gated warning by default); and
- it **shall** be selectable by `--deny I010`, so a project may choose to gate on planned-but-unimplemented tests.

`retired` TestCases **shall not** emit `I010` (a retired test is intentionally decommissioned, not planned).

**Source:** GH issue #6 (optional informational code for planned TestCases).

**Acceptance criteria:** a `status: draft` TestCase with a missing `sourceFile` produces `I010` (and no `W004`); `validate` exits `0` in its presence; `validate --deny I010` exits `2`; a `retired` TestCase produces neither `W004`/`W009` nor `I010`.
