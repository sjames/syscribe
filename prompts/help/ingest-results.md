# ingest-results — ingest external test results (enables W010)

## SYNOPSIS
    syscribe -m <root> ingest-results [--format cargo-json|junit|session-log] <file>

## DESCRIPTION
Parses an external test report and writes the verdict sidecar at
<root>/.syscribe/results.json. Once present, `matrix`/`trace`/`safety-case`/
`testplan` annotate a covered TestCase with its verdict; `validate` additionally
emits W010 for an `active` TestCase whose `testFunctions:` last failed, were
skipped, or were missing from the run.

Two source shapes, feeding two different verdict axes:

- **`cargo-json`/`junit`** — automated test output, reduced to a per-**function**
  verdict (matched against a TestCase's `testFunctions[].function`).
- **`session-log`** — a JSON array of manual/exploratory verification records,
  one per Gherkin scenario actually exercised against a live system (a curl
  session, an MQTT probe client, a CLI walkthrough — anything session-based
  and not itself a `#[test]` function), reduced to a per-(TestCase, scenario)
  verdict. This is what gives a TestCase with **no** `testFunctions:` (the norm
  for a manually-verified one) the same machine-checkable pass/fail surface an
  automated one gets, instead of a hand-typed "Verified live: ..." sentence
  nothing can tell true from asserted. Each record:

  ```json
  {
    "testCase": "TC-WEB-011",
    "scenario": "An empty/unset allowlist denies everything",
    "steps": [
      {"cmd": "curl -X POST ... -d command=restart_hmi", "expect_status": 400}
    ],
    "result": "pass",
    "timestamp": "2026-09-12T08:02:29Z"
  }
  ```

  `scenario` must match a `Scenario:`/`Scenario Outline:` title in that TestCase's
  body exactly. `steps` is opaque (whatever the session actually used) but must
  be present and non-empty. `result` is `pass`/`fail`/`unknown`. Unlike
  `cargo-json`/`junit`'s tolerant line-skipping, a malformed record (empty
  `testCase`/`scenario`/`steps`, or an unrecognized `result`) is a **hard parse
  error** naming the offending record — nothing is written, and the existing
  sidecar (if any) is untouched.

  A TestCase's rolled-up verdict (in `trace`/`matrix`/`safety-case`/`testplan`)
  is `[pass]` only when **every** scenario in its body has a recorded `pass`;
  any recorded `fail` makes the whole TestCase `[fail]`; partial or absent
  coverage leaves it unannotated (unknown) — the same "all-or-fail" rule
  `testFunctions:` verdicts already follow.

## OPTIONS
    --format <fmt>   cargo-json (libtest JSON), junit (JUnit XML), or session-log
                     (manual/exploratory verification). cargo-json/junit are
                     inferred from the file extension if omitted; session-log is
                     never inferred and must always be named explicitly.

## EXAMPLES
    cargo test -- -Z unstable-options --format json | syscribe -m model/ ingest-results --format cargo-json /dev/stdin
    syscribe -m model/ ingest-results results.xml
    syscribe -m model/ ingest-results --format session-log session.json

## SEE ALSO
    validate (W010), matrix, trace
