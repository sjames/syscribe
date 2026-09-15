---
id: REQ-TRS-INGEST-002
type: Requirement
name: Executed-evidence views shall roll up session-log scenario verdicts for TestCases with no testFunctions
status: draft
reqDomain: software
verificationMethod: test
---

For a `TestCase` that declares **no** `testFunctions:` (the norm for one verified only through a
recorded manual/exploratory session), `trace`/`matrix`/`safety-case`/`testplan` **shall** compute
its aggregate executed-evidence verdict from `session-log`-ingested scenario verdicts against
that `TestCase`'s own Gherkin `Scenario:`/`Scenario Outline:` titles, using the exact same
all-or-fail rule already applied to `testFunctions:` verdicts: `Fail` if any scenario's recorded
verdict is `fail`; `Pass` only if **every** scenario in the body has a recorded `pass`; `Unknown`
otherwise (no data, partial coverage, or an `unknown` result). A `TestCase` that **does** declare
`testFunctions:` **shall** continue to be scored against those, unchanged, regardless of any
session-log data also present for it.

This makes a `TestCase` covered only by prose evidence (no ingested result of either kind)
visibly distinguishable, in every consumer's existing text and JSON output, from one covered by
an ingested session-log pass — the same annotation (`[pass]`/`[fail]`, unannotated otherwise)
`cargo-json`/`junit` results already produce, now also sourced from `session-log` data.

**Source:** GitHub issue #113.

**Acceptance criteria:**
- `trace` (and, by the same shared verdict computation, `matrix`/`safety-case`/`testplan`)
  annotates a `TestCase` with no `testFunctions:` as `[pass]` once every one of its Gherkin
  scenarios has a recorded `session-log` pass.
- A recorded `fail` on any one scenario marks the whole `TestCase` `[fail]`.
- Partial scenario coverage (some scenarios recorded, others not) leaves the `TestCase`
  unannotated (not falsely `[pass]`).
- A `TestCase` with `testFunctions:` is unaffected by any `session-log` data also present for it.
