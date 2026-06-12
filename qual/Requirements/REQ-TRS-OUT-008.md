---
id: REQ-TRS-OUT-008
type: Requirement
name: Tool shall ingest external test results and flag unverified passing claims (W010)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** ingest external test-run results and record a per-function verdict so that "verified" can mean "covered by a test that actually passed". Specifically the tool **shall**:

- provide `ingest-results --format <cargo-json|junit> <file>` to parse libtest JSON or JUnit XML and persist a verdict sidecar at `<model_root>/.syscribe/results.json`;
- map each result test to a `testFunctions[].function` by its leaf name (last segment after `::`, `.`, `#`, `/`);
- when a results sidecar is present (or `validate --results <file>` is supplied), emit `W010` for every `active` TestCase whose `testFunctions[].function` last **failed**, was **ignored/skipped**, or was **missing** (not present) in the ingested run;
- emit no `W010` for functions that passed, and no `W010` at all when no results have been ingested.

Because `W010` is a warning, projects can gate on it with `--deny W010` once test results are wired into CI.

**Source:** Issue #4 (ingest test results → per-TestCase pass/fail status)

**Acceptance criteria:** after `ingest-results`, an `active` TestCase whose function failed or did not run produces `W010`; a passing function produces none; `validate --deny W010` then exits `2`. Both `cargo-json` and `junit` formats are supported.
