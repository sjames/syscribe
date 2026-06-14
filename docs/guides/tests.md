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

```yaml
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
    scenario: "Two threads at different priorities; lower-priority never runs first"
  - function: test_preemption_on_unblock
    scenario: "Higher-priority thread unblocked mid-execution preempts the running thread"
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
```

The body **must** contain a `gherkin` fenced block (E011 if absent). This is the machine-
readable test specification that can be compared against actual test code.

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
formal security verification evidence package (see §7.6 below).

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

---
