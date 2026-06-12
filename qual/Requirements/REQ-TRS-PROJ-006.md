---
id: REQ-TRS-PROJ-006
type: Requirement
name: The read-only safety/security analysis commands shall honour the --config projection lens
status: draft
reqDomain: software
verificationMethod: test
---

The read-only analysis and reporting commands **shall** accept the same
configuration lens as `validate --config` ([[REQ-TRS-PROJ-001]]) so that a single
variant can be analysed in isolation, not only the 150% superset. This applies to:

- `metrics` (ISO 26262-5 quantitative HW safety metrics),
- `cyber-risk` (ISO/SAE 21434 risk picture),
- `co-analysis` (safety ↔ security co-analysis),
- `verification-depth` (SIL/ASIL-driven verification-rigour check),
- `safety-case` (GSN safety-case assembly).

(`audit` is covered separately by [[REQ-TRS-OUT-013]].)

### Behaviour

- Given `--config <CONF|features>`, each command **shall** compute its report only
  over the elements **active** in that configuration (per `appliesWhen`, via
  `projection::project` — the same active subset used by `validate --config`). An
  element that is inactive in the selected variant **shall not** appear in, or
  contribute to, the command's output.
- With no `--config`, behaviour **shall** be unchanged (whole-model view).
- When **no feature model** is present, `--config` is **dormant** (falls back to the
  whole-model report) rather than an error.
- An **unresolvable** `--config` argument is a usage error and **shall** exit `1`.

**Source:** ADR-PROJ-001; GH #35 (generalised from `audit --config` to the other
read-only safety/security commands).

**Acceptance criteria:**

- Each of `metrics`, `cyber-risk`, `co-analysis`, `verification-depth` and
  `safety-case` accepts `--config <C>`: a valid `--config` exits `0`, and an
  unresolvable `--config` exits non-zero.
- For a model with a SIL-3 requirement `appliesWhen`-gated to an unselected
  feature, `verification-depth` lists that requirement whole-model but **omits** it
  under `--config <the config that excludes the feature>` (proof the report is
  computed over the projected active subset).
