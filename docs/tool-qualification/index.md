# Tool Qualification

`TOOL QUALIFICATION · ISO 26262 · IEC 61508`

Syscribe qualifies itself for use in safety-critical projects using its own format. The `qual/` directory in the repository **is a Syscribe model** — it contains the tool's requirements and test cases written in exactly the same Markdown/YAML format used for any other system model.

This means the tool being qualified also validates its own qualification evidence.

---

## Why tool qualification?

ISO 26262 Part 8 §11 and IEC 61508 Part 3 Annex D require that software tools used in safety-relevant development activities be qualified before their outputs can be trusted. The level of qualification effort depends on the **Tool Confidence Level (TCL)**:

| TCL | Required evidence |
|---|---|
| TCL1 | Brief description; no dedicated qualification |
| **TCL2** | **Validation evidence: test spec + test report + release note** |
| TCL3 | TCL2 plus development process evidence (architecture, unit tests, anomaly log) |

Syscribe targets **TCL2**. The qualification package is complete in `qual/`.

---

## Structure of qual/

```
qual/
  _index.md                    ← Syscribe Package element (the TRS itself)
  Requirements/                ← 60 REQ-TRS-* requirements
    REQ-TRS-PARSE-001.md
    REQ-TRS-VAL-001.md
    ...
  TestCases/                   ← 69 TC-TRS-* test cases with Gherkin
    TC-TRS-PARSE-001.md
    TC-TRS-VAL-001.md
    ...
  fixtures/                    ← minimal Syscribe models crafted to trigger each condition
    TC-TRS-VAL-001/
      E001/                    ← triggers parse error E001
      E002/                    ← triggers parse error E002
      ...
  tests/
    run_qual.sh                ← discovers TCs from TestCases/, runs each against fixtures/
    lib.sh                     ← assert_has_code(), assert_exit_zero(), ...
    tc/
      TC-TRS-PARSE-001.sh      ← one shell function per TC
      ...
    tvr/
      generate_tvr.sh          ← reads results.ndjson → TVR.md
      TVR.md                   ← Tool Validation Report (generated)
```

The requirements (`REQ-TRS-*`) are native `Requirement` elements with stable IDs, `status:`, `reqDomain:`, and `verificationMethod:`. The test cases (`TC-TRS-*`) are native `TestCase` elements with Gherkin scenarios, `testLevel: L3`, and `verifies:` links back to the requirements.

---

## The self-validating property

Because `qual/` is a Syscribe model, you can run the tool being qualified against its own qualification evidence:

```bash
syscribe -m qual/
```

This produces a standard validation report covering all 60 requirements and 60 test cases. Any structural error in the qualification model — a malformed frontmatter, a dangling `verifies:` reference, a duplicate ID — is caught by the same validation rules the qualification tests exercise.

```
$ syscribe -m qual/

## 1. Executive Summary

| Metric    | Count |
|---|---|
| Total elements | 123   |
| Errors         | 0     |
| Warnings       | 63    |
| Requirements (total) | 60 |
| Test cases           | 60 |
| Gherkin scenarios    | 181 |
```

The 63 warnings are all W005 ("possible orphan") — expected, because the TRS requirements are intentionally root-level with no parent hierarchy.

---

## Running the tests manually

The test runner discovers test cases by reading `qual/TestCases/TC-TRS-*.md` with `find` and extracting frontmatter (id, title, verifies) using `awk`. For each TC it sources the matching shell script in `qual/tests/tc/` and runs one or more `syscribe -m qual/fixtures/...` invocations, asserting on stdout content and exit codes.

```bash
# Build syscribe and run all 60 test cases
./qual/tests/run_qual.sh

# Skip rebuild if the binary is already current
./qual/tests/run_qual.sh --no-build

# Run a single TC by ID (useful when developing a new test)
./qual/tests/run_qual.sh --no-build TC-TRS-VAL-001

# Self-check: validate the TRS model itself
syscribe -m qual/
```

The TVR is written to `qual/tests/tvr/TVR.md` after every run.

Sample output:

```
Building syscribe...
    Finished dev profile in 0.05s
Discovered 60 test cases

[TC-TRS-PARSE-008] Verify that invalid YAML frontmatter produces error E002.
  ▶ valid YAML frontmatter produces no E002
    ✓ no E002 in output
    ✓ exit code 0
  ▶ invalid YAML frontmatter produces E002
    ✓ E002 present in output
    ✓ exit code 1 (non-zero)
  ✓ PASS (4 assertions)

...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Results:  60 total  60 passed  0 failed  0 skipped
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
TVR written to: qual/tests/tvr/TVR.md
```

---

## How each test works

Every test case follows the same pattern:

1. **Fixture** — a minimal Syscribe model in `qual/fixtures/` crafted to trigger (or not trigger) a specific condition. Each scenario has its own subdirectory, so test isolation is complete.

2. **Run** — `syscribe -m qual/fixtures/<scenario>/` captures stdout and the exit code.

3. **Assert** — the shell script checks what was produced:

```bash
# TC-TRS-PARSE-008.sh
tc_TRS_PARSE_008() {
    local F="$1"
    run_scenario "valid YAML produces no E002" "$F/TC-TRS-PARSE-008/valid-yaml"
    assert_no_code  "E002"
    assert_exit_zero

    run_scenario "invalid YAML produces E002" "$F/TC-TRS-PARSE-008/invalid-yaml"
    assert_has_code "E002"
    assert_exit_nonzero
}
```

The assertion library (`lib.sh`) provides:

| Function | What it checks |
|---|---|
| `assert_has_code E002` | Output contains `\| E002 \|` |
| `assert_no_code E002` | Output does not contain `\| E002 \|` |
| `assert_exit_zero` | `$?` is 0 |
| `assert_exit_nonzero` | `$?` is non-zero |
| `assert_output_contains "text"` | Output contains the string |

---

## The document chain

The qualification package produces three TCL2 artefacts:

| Document | Location | Generated by |
|---|---|---|
| **Tool Requirements Spec (TRS)** | `qual/Requirements/` + `qual/TestCases/` | Authored in Syscribe format; validated by `syscribe -m qual/` |
| **Tool Validation Plan (TVP)** | Gherkin `Scenario:` blocks in each TC file | Part of the TC authoring; reviewed by reading the `.md` files |
| **Tool Validation Report (TVR)** | `qual/tests/tvr/TVR.md` | Generated by `run_qual.sh` after each test run |

The TVR records the syscribe version, date, and a pass/fail verdict for every TC. It is the primary evidence document submitted to a safety assessor.

---

## Coverage summary

The 60 test cases cover:

| Area | Requirements | Test cases |
|---|---|---|
| File discovery and frontmatter parsing (§11.1–11.2) | REQ-TRS-PARSE-001–009 | TC-TRS-PARSE-001–009 |
| Qualified name derivation (§11.3) | REQ-TRS-QNAME-001–004 | TC-TRS-QNAME-001–004 |
| Cross-reference resolution (§11.5–11.6, §11.10) | REQ-TRS-XREF-001–005 | TC-TRS-XREF-001–005 |
| Element type handling (§2, §11.4) | REQ-TRS-ELEM-001–003 | TC-TRS-ELEM-001–003 |
| ID scheme validation | REQ-TRS-ID-001–004 | TC-TRS-ID-001–004 |
| Parse-time errors E001–E015, E300–E304 (§11.12) | REQ-TRS-VAL-001 | TC-TRS-VAL-001 |
| Model-time errors E101–E106, E310–E315 (§11.12) | REQ-TRS-VAL-002 | TC-TRS-VAL-002 |
| Warnings W001–W007, W300–W305 (§11.12) | REQ-TRS-VAL-003 | TC-TRS-VAL-003 |
| Integrity-level propagation E841–E843, W808 (§12.7) | REQ-TRS-VAL-004 | TC-TRS-VAL-004 |
| Finding content and severity (§11.7) | REQ-TRS-VAL-005–007 | TC-TRS-VAL-005–007 |
| Enum fields E019–E022, ASIL warnings W701–W703, W008 | REQ-TRS-VAL-008 | TC-TRS-VAL-008 |
| Allocation E500–E503, View W500–W502, docs W600–W601 | REQ-TRS-VAL-009 | TC-TRS-VAL-009 |
| Traceability rules §12 (E310–E315, W300–W305) | REQ-TRS-TRACE-001–008 | TC-TRS-TRACE-001–008 |
| Acyclicity of hierarchical relationships (E016–E018) | REQ-TRS-TRACE-009 | TC-TRS-TRACE-009 |
| Configuration element E200–E201, E209 | REQ-TRS-CONF-001 | TC-TRS-CONF-001 |
| Diagram element E400–E402, W400–W412 | REQ-TRS-DIAG-001 | TC-TRS-DIAG-001 |
| HazardousEvent E800–E804, E833–E836, W800 | REQ-TRS-SAFE-001 | TC-TRS-SAFE-001 |
| SafetyGoal E805–E806, E825, E837, W801, W805–W806 | REQ-TRS-SAFE-002 | TC-TRS-SAFE-002 |
| DamageScenario / ThreatScenario E807–E814, E826 | REQ-TRS-SAFE-003 | TC-TRS-SAFE-003 |
| Cybersecurity elements E815–E824, E827–E832, W802–W804, W807 | REQ-TRS-SAFE-004 | TC-TRS-SAFE-004 |
| FaultTree / FaultTreeGate / FaultTreeEvent E900–E909, W900–W901 | REQ-TRS-FTA-001 | TC-TRS-FTA-001 |
| FMEASheet / FMEAEntry E911–E914, W902–W904 | REQ-TRS-FMEA-001 | TC-TRS-FMEA-001 |
| TARASheet E940–E941, W905 | REQ-TRS-TARA-001 | TC-TRS-TARA-001 |
| Output format and exit codes | REQ-TRS-OUT-001–005 | TC-TRS-OUT-001–005 |
| CLI interface | REQ-TRS-CLI-001–003 | TC-TRS-CLI-001–003 |

---

## Validation gaps found during qualification

Running the test suite against an earlier version of syscribe revealed six implementation gaps that were fixed as part of producing a passing suite:

| Gap | Fix |
|---|---|
| Unknown `type:` values silently loaded as "Other" | Implement E005 in validator |
| Files without `---` emitted W008 instead of E001 | Add `ParseIssue::NoFrontmatter` to walker; emit E001 |
| YAML parse failures emitted W008 instead of E002 | Add `ParseIssue::YamlError` to walker; emit E002 |
| `Scenario Outline:` without `Examples:` not caught (E014) | Fix block-close check in Gherkin validator |
| Cross-domain `supertype:` links not enforced (E315) | Implement domain check in model-time validator |
| Invalid model path exited 0 instead of non-zero | Add path existence check in `main.rs` |

This is tool qualification working as intended: the test suite surfaces gaps, the gaps are fixed, and the suite is re-run to confirm.

---

## CI/CD integration

The qualification suite runs automatically on every push to `main` and on every pull request that touches `crates/` or `qual/`. The workflow is at `.github/workflows/qual.yml`.

```yaml
jobs:
  qualify:
    name: TCL2 qualification suite (69 TCs)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Build syscribe
        run: cargo build --package syscribe

      - name: Validate TRS model (qual/ self-check)
        run: ./target/debug/syscribe -m qual/

      - name: Run qualification test suite
        run: bash qual/tests/run_qual.sh --no-build

      - name: Upload Tool Validation Report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: TVR-${{ github.sha }}
          path: qual/tests/tvr/TVR.md
```

The workflow uploads the TVR as a build artifact on every run — including failures — so evidence is always available at the commit SHA. A failed run blocks merging; a green run means the qualification suite passed against the exact binary that was built from the PR.

### What CI checks

Every CI run performs two independent checks:

1. **`syscribe -m qual/`** — validates the TRS model itself. If a requirement or test case has malformed frontmatter, a dangling `verifies:` reference, or a duplicate ID, this step fails before any tests run.

2. **`bash qual/tests/run_qual.sh`** — runs all 60 test cases against the newly built binary. Each TC invokes the binary against a crafted fixture and asserts on stdout and exit code. A single failing assertion causes the run to exit 1.

### Using the TVR artifact as evidence

After a green CI run, download `TVR-<sha>.md` from the workflow artifacts page. This document records:

- The exact binary version under test
- The date and commit SHA
- A pass/fail verdict for every one of the 60 test cases

This is the primary evidence document for a TCL2 qualification submission. Archive it alongside the release binary.

---

## Extending the qualification

To add a new requirement:

1. Create `qual/Requirements/REQ-TRS-<AREA>-NNN.md` with `type: Requirement`, a stable `id:`, and a SHALL body.
2. Create `qual/TestCases/TC-TRS-<AREA>-NNN.md` with `type: TestCase`, Gherkin scenarios, and `verifies: [REQ-TRS-<AREA>-NNN]`.
3. Add `qual/fixtures/TC-TRS-<AREA>-NNN/<scenario>/` with the minimal model for each scenario.
4. Create `qual/tests/tc/TC-TRS-<AREA>-NNN.sh` with the `tc_TRS_<AREA>_NNN()` function.
5. Run `syscribe -m qual/` to validate the new TRS elements, then `./qual/tests/run_qual.sh` to run the tests.
