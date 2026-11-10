# ingest-results — ingest external test results (enables W010)

## SYNOPSIS
    syscribe -m <root> ingest-results [--format cargo-json|junit|session-log|records|robot|cucumber|plugin:<alias>] [--run <run-id>] [--config <CONF-id>] [--replace] [--strict] [--environment <TE-id>] [--build-commit <sha>] [--operator <name>] [--artefact <file>]... [--utc-offset <Z|+HH:MM>] <file>

`--run <id>` additionally retains the ingest under a run identity for `results diff` (see `results`).

`--config <CONF-id>` records the ingest as the evidence of one Configuration only (see DESCRIPTION).

## DESCRIPTION
Parses an external test report and writes the verdict sidecar at
<root>/.syscribe/results.json. Once present, `matrix`/`trace`/`safety-case`/
`testplan` annotate a covered TestCase with its verdict; `validate` additionally
emits W010 for an `active` TestCase whose `testFunctions:` last failed, were
skipped, or were missing from the run.

If the project protects `.syscribe/**` (`[agents] protected_paths`), an agent's ingest is refused with exit 1 unless
`ingest-results` is named in `[agents] sidecar_writers`; the refusal says so. A person sets that, never the agent
(`syscribe help policy`, section "Protecting recorded evidence"). To keep `validate` reporting a hand-edited sidecar,
commit the ingest with `--commit` and set `sidecar_audit = "command"`.

`--config <CONF-id>` must resolve to a `Configuration` (else exit 1 and nothing is written).
The results are kept under that configuration only — the global verdicts are untouched — so
a test that fails only on one variant is no longer one global `fail`. A configuration is judged
on its *effective* results: the global ones overlaid with its own (its own verdict wins,
otherwise the global one applies; no configuration inherits another's). Each `matrix` column
uses its own, and under `--config CONF-X` `validate`, `audit`, `trace`, `safety-case`,
`coverage tree`, `matrix --rollup` and the other evidence readers do too. With `--run`, the run
keeps the configuration sections; `results diff` names a changed test `<test> @ <config>`.
A syscribe older than this feature ignores the configuration sections and drops them when it
rewrites the sidecar (`ingest-results`), so keep tool versions consistent.

Two source shapes, feeding two different verdict axes:

- **`cargo-json`/`junit`** — automated test output, reduced to a per-**function**
  verdict (matched against a TestCase's `testFunctions[].function`). JUnit results
  are also keyed by `classname::name`, so a reference like `com.A#test_x` (or
  `com.A.test_x`, or `com.A::test_x`) is exact
  and a bare `test_x` takes the worst of any same-named tests; a testcase with a
  `flakyFailure`/`rerunFailure` child and no failure is `flaky` (not a pass; `W010`, and
  `W615` for an approved plan). A sidecar that contains `flaky` cannot be read by an
  older syscribe (it ignores the whole sidecar), so keep tool versions consistent.
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

After a function-level ingest, a line `Expected functions: N; not run: M missing,
K skipped` (with the first names) summarises the `testFunctions` of native `active`
TestCases that the report lacks (missing) or reports as skipped. For JUnit, the
`message` attribute of a failure/error/skipped/flakyFailure child and the testcase
`time` are kept for every non-passing case (see `results failures`).

### robot (Robot Framework output.xml)

`--format robot` reads a Robot Framework result file (Robot 6 and 7; an `.xml` file whose root is `<robot>` is
recognised without `--format`). A test is joined to a TestCase by a **tag**: a tag equal to a TestCase id or
qualified name, or `extref:<value>` for an `extRef` (any case). Other tags (requirements, `smoke`) are ignored.
A test named exactly like a `Scenario:` title of the TestCase is recorded against that scenario; any other test
is the verdict of the whole TestCase. A test whose tags name no TestCase is counted and reported as unmatched,
never dropped (`--strict` rejects such a file).

Verdicts: `PASS` is pass; `FAIL` is fail, keeping Robot's message; `SKIP` and `NOT RUN` are not-run. A test that
never ran because a parent suite's setup failed (Robot reports it as FAIL) is `blocked`, since nothing about the
product was tested; a test whose own setup, body or teardown failed is fail. Several tests for one TestCase and
scenario count as one: the worst verdict decides, the first failing test and its message lead the detail, and
the times span all of them. Keyword-level results, logs and screenshots are not read. RPA results (tasks) are
refused.

Robot records the tool (`Robot 7.4.2`) and local start and end times with no zone: the times are kept as
written with the zone you state, `--utc-offset +02:00` (default `Z`). The build, rig and operator are not in the
file: pass `--build-commit`, `--environment` and `--operator`. `--artefact <file>` (repeatable) references a
file, such as Robot's log, by its blake3 hash; the file is never copied.

### cucumber (Cucumber JSON)

`--format cucumber` reads the report of Cucumber's classic `json` formatter (Cucumber-JVM, Cucumber.js,
Cucumber-Ruby; a `.json` file shaped like one is recognised without `--format`). The newer NDJSON message
protocol is a different format and is refused with a message that says so.

One record per scenario. A scenario is joined to a TestCase by a **tag** (its own or its feature's): a TestCase
id or qualified name, or `@extref:<value>` for an `extRef` (any case). A scenario titled like a `Scenario:` of
that TestCase is recorded against it, any other as the TestCase's own verdict; one that names no TestCase is
counted and reported as unmatched (`--strict` rejects the file).

A scenario is `fail` when a step is `failed` or `ambiguous` or a hook failed (the first failing step and the
first line of its message are kept), `not-run` when a step is `pending`, `undefined` or `skipped` or the
scenario has no steps of its own, otherwise `pass`. A background counts toward the scenarios after it, whether
the report writes it as a separate element (JVM, Ruby) or merges it into each scenario (JS), and its hooks too.
The rows of a Scenario Outline are recorded as `<title> [row N]` and the outline's own title carries the worst
row. If a scenario appears more than once (a retry plugin writes every attempt) the last attempt decides, and a
pass after a failure is `flaky`, never `pass`. Several scenarios for one TestCase and scenario fold into one
record, the worst verdict deciding and the failing scenario first. Cucumber.js writes only a retried scenario's
last attempt, so it cannot show a flaky one.

A start timestamp (`start_timestamp`, where the formatter writes one) and the step durations give the time span.
Embedded screenshots and step output are never stored. `--environment`, `--build-commit`, `--operator` and
`--artefact` supply what the report does not contain.

### plugin:<alias> (any other report format)

`--format plugin:<alias>` converts the report with an external program configured as `[plugins.<alias>]` with
`kind = "results"` in `.syscribe.toml`, and stores what it returns as `--format records` would, with the same
validation, join and `--strict`/`--replace`/`--environment`/`--build-commit`. A plugin that is missing, fails,
times out or writes something invalid is `E553` and nothing is written. See `syscribe help plugins`.

### records (system, integration and rig tests)

`--format records` reads a JSON array of normalised result records (REQ-TRS-RESREC-001), for the
results that are not JUnit: system tests, integration tests, hardware-in-the-loop runs. It is never
inferred from the file name.

    [{"testCase": "TC-SYS-012",
      "scenario": "Brake request reaches the actuator in time",
      "verdict": "pass",
      "measurements": [{"name": "latency", "value": "4.2 ms", "limit": {"op": "<", "value": "5 ms"}}],
      "provenance": {
        "build": {"commit": "abc1234", "version": "1.2.3"},
        "environment": "TE-HIL-01", "operator": "alice",
        "startedAt": "2026-10-01T08:00:00Z", "finishedAt": "2026-10-01T08:00:05Z",
        "artefacts": [{"uri": "s3://rig/run42/log.txt", "hash": "sha256:<64 hex digits>"}]}}]

`testCase` is a TestCase id, a qualified name or an `extRef` value of a TestCase (in that order; `extRef`
is unique per model, so a test tool's own identifier such as `RIG:SYS-012` works). The record is stored under
the TestCase's id. `scenario` is a `Scenario:` title of that TestCase; leave it out and the record is the
verdict of the whole TestCase.

Verdicts: `pass`, `fail`, `blocked` (could not run: the rig was down), `not-run`, `inconclusive`,
and `waived`, which needs a non-empty `rationale`. Only `pass` is a pass. A TestCase whose scenarios
all passed or were waived, with at least one waived, shows `[waived]` in `trace`/`matrix`/`safety-case`,
never green; the safety case counts it as not complete. `fail` wins over everything.

A measurement is a quantity with its unit; with a `limit` (`<`, `<=`, `>`, `>=`, `==`) it is checked with the
unit rules of the expression language (`4000 us` is below `5 ms`; `5 m` is not comparable). A `pass`
whose own measurement violates its limit is rejected: the tool never rewrites a verdict.

Provenance is optional. `environment` must be a TestEnvironment of the model (an id or a qualified name,
stored by id). Artefacts are referenced by URI and `sha256:` or `blake3:` hash, never copied. Timestamps
are RFC 3339. Free text is stored with control characters removed.

A `testCase` that is ambiguous, or names an element that is not a TestCase, rejects the whole file and writes
nothing. One that names nothing is reported with a count and the first few keys, kept under `unmatched`, listed
by `results failures`, and counts as evidence for nothing; `--strict` rejects such a file instead.

The rig a result was made on is `provenance.environment`, or `--environment` for records that name none.
Results are kept per rig, and a TestPlan with `runsOn:` judges only the rigs it lists (see `testplan`): all
must pass, and a rig with no passing result leaves the plan `incomplete`. A plan without `runsOn:` takes the
worst over every rig. Results from a retired or out-of-calibration rig do not count as a pass.

Each stored record keeps a hash of its TestCase and of the requirements it verifies. If either changes
afterwards, `validate` raises `W091` for the TestCase (gate with `--deny W091`) and `trace` marks the verdict
`[stale]`; the verdict itself is not altered. Re-run the test and re-ingest to clear it. Hashes a report
claims for itself are discarded.

One file is one run: if it holds several records with the same test, scenario and environment, the last
is kept and the ingest says how many were superseded. A leading byte-order mark is ignored.

Like `session-log`, a malformed record (unknown verdict or field, `waived` without a rationale, a bad
hash or timestamp, an inconsistent measurement, an unknown environment, an empty array) is a hard error
naming the record. Nothing is written and an existing sidecar is untouched.

Records are kept in the sidecar's `records` section, the latest per (test case, scenario, environment). An
ingest supersedes the records with the same key and keeps the others, so a hardware-in-the-loop ingest
does not erase the bench results; `--replace` drops the records it does not contain. Function-level and
session-log verdicts are untouched. `--run` and `--config` work as for the other formats.

## MERGE SEMANTICS
The sidecar holds two independent sections, and each ingest **merges** into the
existing sidecar, replacing only its own section:

- a `cargo-json`/`junit` ingest replaces the function-level verdicts (`by_leaf`)
  and keeps any session-log verdicts;
- a `session-log` ingest replaces the scenario verdicts (`by_scenario`) and keeps
  any function-level verdicts.

So ingesting a `cargo test` run and a manual session log, in either order, leaves
both in effect. Re-ingesting the same kind still **replaces** that section (a new
run supersedes the last one; results never accumulate across runs). A missing or
unreadable existing sidecar is treated as empty; a malformed new input writes
nothing and leaves the existing sidecar untouched.

The top-level `format`/`source`/`count`/`ingested_at_unix` fields describe the
latest ingest; `leaf_meta`/`scenario_meta` record which report each section came
from. Sidecars written by earlier versions are still read.

## OPTIONS
    --run <id>       also retain the ingest under this run identity for `results diff`
    --config <CONF-id>  record the ingest as the evidence of this Configuration only
    --strict         records only: reject the whole file if any record names no TestCase
    --operator <name>  records, robot, cucumber, plugins: the operator for every record that names none
    --artefact <file>  records, robot, cucumber, plugins (repeatable): reference this file by its hash, never copied
    --utc-offset <Z|+HH:MM>  robot only: the zone of Robot's zone-less timestamps (default Z)
    --build-commit <sha>  records and plugins: the commit for every record that has none
    --environment <TE-id>  records only: the rig for every record that names none
                     (a record's own provenance.environment is kept); must be a TestEnvironment
    --replace        records only: drop the stored records this ingest does not contain,
                     instead of superseding only those with the same test, scenario and
                     environment
    --format <fmt>   cargo-json (libtest JSON), junit (JUnit XML), session-log
                     (manual/exploratory verification) or records (normalised result
                     records). cargo-json/junit are inferred from the file extension if
                     omitted; session-log and records are never inferred and must always
                     be named explicitly.

## EXAMPLES
    cargo test -- -Z unstable-options --format json | syscribe -m model/ ingest-results --format cargo-json /dev/stdin
    syscribe -m model/ ingest-results results.xml
    syscribe -m model/ ingest-results --format session-log session.json
    syscribe -m model/ ingest-results --format records rig-run.json
    syscribe -m model/ ingest-results --format records --replace rig-run.json
    syscribe -m model/ ingest-results --format plugin:rig --environment TE-HIL-01 rig.log
    syscribe -m model/ ingest-results output.xml --environment TE-HIL-01 --build-commit abc1234   # Robot
    syscribe -m model/ ingest-results cucumber.json --environment TE-HIL-01                       # Cucumber

## SEE ALSO
    validate (W010, W091), matrix, trace, results (failures lists unmatched records)
