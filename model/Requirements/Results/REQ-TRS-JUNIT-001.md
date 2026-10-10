---
type: Requirement
id: REQ-TRS-JUNIT-001
name: "JUnit ingestion keys tests by class, and records flaky outcomes"
status: draft
reqDomain: software
reqClass: system
tags:
  - results
---

JUnit ingestion shall keep the information that distinguishes tests and outcomes (GH #259).

## Behavior

- A `<testcase classname=C name=N>` is recorded under its qualified key `C::N` as well as under its leaf `N`. A `testFunctions[].function` reference with a class separator (`C#N`, `C::N`, `C.N`) is looked up by the qualified key first and falls back to the leaf.
- When two classes have a test of the same leaf name, the leaf entry holds the worst verdict (Fail, then Flaky, then Pass, then Ignored) so a leaf-only reference is conservative; qualified references stay exact.
- A testcase with a `<flakyFailure>`, `<flakyError>`, `<rerunFailure>` or `<rerunError>` child and no `<failure>`/`<error>` is a new `flaky` verdict, not a pass. `flaky` is not a pass for a TestCase, and `W010` reports it ("passed only after a retry"), so it can be gated with `--deny W010`.
- A testcase with both a failure and rerun children is a Fail.
- `ingest-results` reports the flaky count.
- An approved TestPlan with a flaky member raises `W615` (flaky is not a pass).
- Limitation: a sidecar containing `flaky` is unreadable by an older syscribe, which then ignores the whole sidecar.
