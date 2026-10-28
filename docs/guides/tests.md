# Part IV — Tests

`GUIDE · PART IV — TESTS`


### 4.1 The test pyramid

syscribe supports five test levels that map to standard V-model practice:

| Level | `testLevel:` | Typical use | Equivalent V-model stage |
|---|---|---|---|
| L1 | `L1` | Unit tests, host-side | Module testing |
| L2 | `L2` | Property / fuzz tests | Module testing (automated) |
| L3 | `L3` | Formal proofs (Kani, etc.) | Formal analysis |
| L4 | `L4` | Integration / QEMU / simulation | Integration testing |
| L5 | `L5` | Hardware-in-the-loop | Validation on target |

### 4.2 Authoring a TestCase

````markdown
---
type: TestCase
id: TC-SCHED-001
name: "Scheduler selects highest-priority ready thread"
status: active
testLevel: L1
verifies:
  - REQ-SCHED-001
sourceFile: tests/host/src/scheduler_tests.rs
testFunctions:
  - function: test_priority_ordering
    scenario: "Higher priority thread runs first"
  - function: test_preemption_on_unblock
    scenario: "Preemption on unblock"
tags: [safety, scheduler]
---

```gherkin
Feature: Scheduler priority ordering

  Scenario: Higher priority thread runs first
    Given threads A (priority 5) and B (priority 10) are both ready
    When the scheduler selects the next thread
    Then thread B is selected

  Scenario: Preemption on unblock
    Given thread A (priority 5) is running
    When thread B (priority 10) is unblocked
    Then a context switch to B occurs immediately
```
````

The body **must** contain a `gherkin` fenced block (E011 if absent). This is the machine-
readable test specification that can be compared against actual test code.

### Verifying architecture elements directly (`verifiesElement:`)

A unit-level test often targets a component, not a requirement. A `TestCase` may name the `PartDef`, `InterfaceDef`, `ActionDef` or `StateDef` it exercises with `verifiesElement:` (qualified names or stable ids, string or list):

```yaml
type: TestCase
id: TC-SCHED-002
name: "Run queue bitmap operations"
status: active
testLevel: L1
verifiesElement:
  - Scheduler::RunQueue
```

An unresolved entry is `E102`; an entry that is not one of those four kinds is `E129`. Under `--config`, a test naming an element the variant removes is `W019`, as for `verifies:`.

An active, non-abstract `PartDef`, `InterfaceDef`, `ActionDef` or `StateDef` is *tested* when an active `TestCase` names it in `verifiesElement:`, or when it `satisfies:` a requirement that an active `TestCase` verifies. Otherwise `validate` raises `W104`. Draft and deprecated elements are not checked. Where an element is deliberately untested (a pure data holder, a wrapper covered by its parts), record why with `testExempt: <rationale>` on it. To make untested elements fail CI, run `validate --deny W104`.

### 4.3 TestPlan

A `TestPlan` groups test cases for a specific verification objective — a configuration,
a scope, or a certification evidence package. It is the syscribe equivalent of a test
specification document.

```yaml
---
type: TestPlan
id: TP-SAFETY-001
name: "Kernel Safety Verification Plan — ASIL D"
status: approved
scope: certification
configurations: [CONF-M33-QEMU, CONF-RP-PICO2-HIL]
demonstrates:
  - Safety::SG-KERNEL-001-SchedulingIntegrity
  - Safety::SG-KERNEL-002-SpatialIsolation
selection:
  tags: [safety]
  testLevels: [L1, L4, L5]
---

This plan defines the minimum test evidence required to support the ASIL D claim
on the kernel scheduling and memory protection subsystems.
```

The `demonstrates:` field links the plan to the safety goals it provides evidence for.
`selection:` is a query — all test cases matching the tags/levels are automatically members.
Empty effective sets trip W612.

Inspect plan membership and lens the matrix through it:

```bash
syscribe -m model testplan TP-SAFETY-001          # list effective members
syscribe -m model matrix --plan TP-SAFETY-001     # coverage filtered to this plan
```

### 4.4 Security test methods (ISO/SAE 21434 §13.3)

Security-relevant test cases carry a `securityTestMethod:` to record the 21434-mandated
test technique:

```yaml
securityTestMethod: penetration_test   # or: fuzz · security_regression ·
                                       #    vulnerability_scan · threat_modeling
```

W809 fires if the method is set to an unrecognised value.

Use a `TestPlan` with `scope: security` and `demonstrates:` pointing to your CSGs as the
formal security verification evidence package (see §7.8 of the ISO/SAE 21434 guide, `iso-21434.md`).

### 4.5 Coverage and gap analysis

```bash
# Which requirements have no test coverage in any configuration?
syscribe -m model matrix --gaps-only

# How many distinct test levels cover each SIL4 requirement?
syscribe -m model verification-depth --sil 4

# Gate: fail if any SIL4 requirement has fewer than 2 distinct test levels
syscribe -m model verification-depth --sil 4 --min-levels 2

# Who covers a specific requirement?
syscribe -m model who-verifies REQ-SCHED-001

# Full traceability chain for a requirement
syscribe -m model trace REQ-SCHED-001
```

**For auditors**: `verification-depth` directly answers "does each SIL/ASIL requirement have
independent multi-level verification?" — a question that traditionally required manually
cross-referencing test plans in DOORS.

### 4.6 Ingesting test results

Feed actual CI test results into syscribe so the matrix shows *passing* coverage, not just
*linked* coverage:

```bash
# After running tests, ingest results
syscribe -m model ingest-results --format cargo-json test-output.json

# Matrix now distinguishes:
#   ✓  covered and passing
#   ▣  linked but last run did not pass
#   ✗  gap (no test case)
```

This turns the matrix from a static traceability view into a live verification dashboard.

### 4.7 Manual/exploratory verification (`session-log`)

Not every scenario fits a `#[test]` function — verifying something that requires a live
end-to-end session (server restarts, requests issued as different roles, a throwaway probe
client, reading back stored rows) produces evidence `cargo test`/CI can't run standalone. Record
it as a `session-log` instead of a hand-typed "Verified live: ..." sentence nothing can check:

```json
[
  {
    "testCase": "TC-WEB-011",
    "scenario": "An empty/unset allowlist denies everything",
    "steps": [
      {"cmd": "curl -X POST .../admin/devices/x/commands -d command=restart_hmi", "expect_status": 400}
    ],
    "result": "pass",
    "timestamp": "2026-09-12T08:02:29Z"
  }
]
```

```bash
syscribe -m model ingest-results --format session-log session.json   # always explicit --format
```

### System and integration tests (`--format records`)

Results from a rig, a hardware-in-the-loop run or an integration suite are rarely JUnit. Write them as
normalised records, with the facts an audit asks for: which build, which rig, what was measured, where
the raw log is.

```json
[{"testCase": "TC-SYS-012", "scenario": "Brake request reaches the actuator in time", "verdict": "pass",
  "measurements": [{"name": "latency", "value": "4.2 ms", "limit": {"op": "<", "value": "5 ms"}}],
  "provenance": {"build": {"commit": "abc1234"}, "environment": "TE-HIL-01",
                 "startedAt": "2026-10-01T08:00:00Z", "finishedAt": "2026-10-01T08:00:05Z"}}]
```

```bash
syscribe -m model ingest-results --format records rig-run.json
syscribe -m model ingest-results --format records --replace rig-run.json   # drop records not in this file
```

**Cucumber.** Tag each scenario with the TestCase it evidences (`@TC-SYS-012`, or `@extref:RIG:SYS-012`), or tag the
feature to cover all its scenarios, and write the report with the `json` formatter (`cucumber-js --format
json:report.json`, `--format json` in Ruby and JVM). `--format cucumber` is optional, since the report is
recognised by its shape. A scenario titled like a `Scenario:` of the TestCase is recorded against it, the rows of
a Scenario Outline are `<title> [row N]` with the outline's title carrying the worst, a failed or ambiguous step
or failing hook fails it, and pending, undefined or skipped steps leave it `not-run`. If your runner retries,
JVM and Ruby write every attempt and a pass after a failure is recorded as `flaky`; Cucumber.js writes only the
last attempt. Worked example, with reports from all three: `examples/results-cucumber/`.

**Robot Framework.** Tag each Robot test with the TestCase it evidences (`[Tags]    TC-SYS-012`, or
`extref:RIG:SYS-012` for the TestCase's `extRef`) and ingest the `output.xml`; `--format robot` is optional
because the file is recognised by its `<robot>` root. A test named exactly like a `Scenario:` title is recorded
against that scenario, any other test as the TestCase's own verdict, and several tests for one TestCase count
as one (the worst decides). Tags that are not TestCases (`REQ-…`, `smoke`) are ignored, and a test with no
matching tag is reported as unmatched. Give the rig and build on the command line
(`--environment TE-HIL-01 --build-commit abc1234`); Robot's times carry no zone, so state it with `--utc-offset`.
A worked example is in `examples/results-robot/`.

Name the test by its TestCase id, or by the identifier your tool already uses: put it in the TestCase's
`extRef:` (for example `extRef: "RIG:SYS-012"`) and use that value as `testCase`. A record that names no
TestCase is reported and counts for nothing; `--strict` rejects the file. When the TestCase, or a requirement
it verifies, is edited after the result was recorded, `validate` warns `W091` and `trace` shows `[stale]`:
re-run the test and ingest again.

Use `blocked` when the rig was unavailable and `not-run` when a test was not executed; neither is a pass.
Use `waived` only with a `rationale`: it shows as `[waived]` and is never counted as passing. Results from
another rig are kept beside this one unless you pass `--replace`. A worked example is in
`examples/results-records/`.

`scenario` must match a `Scenario:`/`Scenario Outline:` title in that `TestCase`'s body exactly.
A malformed record (empty `steps`, an unrecognized `result`) is a hard error — nothing is
written, unlike `cargo-json`/`junit`'s tolerant line-skipping. Once ingested, a `TestCase` with
**no** `testFunctions:` (the norm for a manually-verified one) gets the same `[pass]`/`[fail]`
annotation in `trace`/`matrix`/`safety-case`/`testplan` an automated one gets — `Pass` only once
every scenario in its body has a recorded pass, `Fail` if any recorded a fail.

---
