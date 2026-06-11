---
id: REQ-TRS-VAL-014
type: Requirement
title: Tool shall apply W004/W009 to a TestCase only when its status is active
status: draft
reqDomain: software
verificationMethod: test
---

The source-drift checks `W004` (sourceFile does not exist) and `W009` (testFunction does not resolve) **shall** be emitted for a `TestCase` only when its `status` is `active` — the status that asserts the test currently exists. For every non-live TestCase status (`draft`, `review`, `approved`, `retired`), the tool **shall** suppress `W004` and `W009`, because such a TestCase's `sourceFile`/`testFunctions` may legitimately not exist yet (a specified-but-future verification skeleton) or no longer exist (`retired`).

This scoping **shall not** affect:

- `W004` on non-`TestCase` elements that carry a `sourceFile:` — those are checked regardless of any status; and
- any other validation rule.

> Note: the issue motivating this requirement refers to "active (and verified)"; `verified` is a *Requirement* status, whereas the `TestCase` status vocabulary is `draft | review | approved | active | retired`. The single live status for a TestCase is `active`, consistent with the test-result coverage rule `W010`.

**Source:** GH issue #6 (suppress sourceFile/function drift for non-active TestCases); §11.12 (`W004`, `W009`).

**Acceptance criteria:** a `status: draft` TestCase with a missing `sourceFile` or an unresolved `testFunctions[].function` produces **no** `W004`/`W009`; an otherwise-identical `status: active` TestCase still produces `W004`/`W009`; a non-`TestCase` element with a missing `sourceFile` still produces `W004` whatever its status.
