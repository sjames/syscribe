---
type: Requirement
id: REQ-TRS-JUNITDET-001
name: "ingest retains JUnit failure details and reports expected functions that did not run"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - evidence
---

JUnit ingestion shall keep the evidence needed to explain a non-passing result, and `ingest-results` shall summarise which expected test functions did not run (GH #259, remaining items 3 and 4; `<properties>` and session-log steps stay out of scope).

## Behavior

- For every JUnit testcase that is not a pass (failed, skipped or flaky) the sidecar keeps `details[<leaf>]` (and the class-qualified key) with the `message` attribute of its `failure`/`error`/`skipped`/`flakyFailure` child and the testcase `time` (seconds). Passing testcases add no entry. The field is absent from a sidecar without such cases, and an existing sidecar without it still reads. A session-log ingest keeps the details of the function-level section; a function-level ingest replaces them.
- `results failures [--json]` lists the failing, skipped and flaky functions of the latest ingest with message and time.
- After a function-level ingest with a loaded model, `ingest-results` prints one summary line `Expected functions: N; not run: M missing, K skipped` followed by the first missing and skipped names, counting the `testFunctions` of native `active` TestCases. No line is printed when the model has no such functions.
