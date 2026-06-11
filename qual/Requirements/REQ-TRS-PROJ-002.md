---
id: REQ-TRS-PROJ-002
type: Requirement
title: validate --config shall re-run the full validation rule set over the projected variant
status: draft
reqDomain: software
verificationMethod: test
---

`validate --config C` **shall** re-run the validation rule set over the projected active subset ([[REQ-TRS-PROJ-001]]) so that a variant can be certified on its own — not only the 150% superset.

### Behaviour

- All cross-element rules **shall** be evaluated in the lens, including at least:
  - coverage (an active requirement at `approved`/`implemented` covered by an **active** TestCase; the per-configuration `W015` is the lens-native form);
  - §12 traceability (leaf assignment `W300`, domain match `E313`, parent-not-satisfied `E312`, breakdown-ADR `E310`/`E311`) over active elements;
  - cross-reference resolution (`E102`–`E106`) over active elements — this is where escaping references surface ([[REQ-TRS-PROJ-003]]);
  - safety/security obligations (e.g. `E841`–`E843`, `W806`) over active elements.
- A rule whose inputs are entirely outside the projection **shall not** fire (it concerns a different variant).
- The exit-code contract is unchanged (`0` clean · `1` errors · `2` gated warnings).
- Whole-model `validate` (no `--config`) **shall** be unchanged.
- Parse-time / ID rules already hold for the superset; re-running them on a subset is harmless and need not be specially handled.

**Source:** ADR-PROJ-001; §11.12, §12.

**Acceptance criteria:** A requirement active in C but verified only by a TestCase **inactive** in C is reported uncovered under `validate --config C` (even though it is covered in the 150% model); an active leaf requirement with no **active** satisfying element raises the §12 finding in the lens; the same model's whole-model `validate` (no `--config`) reports neither; the exit code reflects the lens findings.
