---
id: REQ-TRS-PLAN-003
type: Requirement
name: TestPlan Membership (Explicit and Selection)
title: Tool shall compute a TestPlan's effective TestCase set from explicit members plus an additive selection query
status: draft
reqDomain: software
verificationMethod: test
---

A TestPlan's **effective TestCase set** is the union of its explicitly named members and
the TestCases matched by an optional `selection:` query, deduplicated by id:

```
effective(plan) = testCases ∪ match(selection)   (deduped by id)
```

`selection` is **additive only** — it may broaden the set but **never** removes a
TestCase named in `testCases:`.

### Explicit members (`testCases:`)

- Each `testCases:` entry **shall** resolve to a `TestCase`; an entry that does not
  **shall** raise `E601`.
- A `TestCase` named explicitly in `testCases:` whose `status` is `draft` or `retired`
  is still a member (explicit naming is authoritative) but **shall** raise `W613` (a
  not-ready TestCase has been pinned into the plan).

### Selection query (`selection:`)

`selection` has optional sub-fields. An **absent** sub-field is **no constraint**. A
`selection` block that has **no sub-fields at all** matches **nothing** (not
everything):

- `testLevels` — a subset of `L1`–`L5`; a value outside this set **shall** raise `E602`.
- `domains` — a subset of `system | hardware | software`, derived **transitively** from
  a candidate TestCase's `verifies:` targets' `reqDomain:`; a value outside this set
  **shall** raise `E605`.
- `tags` — matched against TestCase `tags`.
- **Draft TestCases are not swept by `selection`** — only explicit naming can pull a
  draft TestCase into the plan.

### Empty plan

- A plan whose **effective set is empty** (no resolvable explicit members and no
  selection match) **shall** raise `W612`.

**Source:** GH #38.

**Acceptance criteria:** the effective set equals `testCases ∪ selection-matches`
deduped by id; an unresolvable `testCases:` entry raises `E601`; a `testLevels` value
outside L1–L5 raises `E602`; a `domains` value outside system/hardware/software raises
`E605`, and `domains` is matched transitively through `verifies:` → `reqDomain:`; a
`selection` with no sub-fields matches no TestCases; a draft TestCase is excluded from a
selection match but included (with `W613`) when named explicitly; a retired explicitly
named TestCase raises `W613`; a plan with an empty effective set raises `W612`.
