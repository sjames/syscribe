---
id: REQ-TRS-AW-001
type: Requirement
name: Tool shall set or modify an element's appliesWhen via CLI, validating the feature model on set
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a CLI command to **add, modify, or remove** the `appliesWhen:`
field of a model element (a `Requirement`, a `Package`, a `Part`/`PartDef`, or any other
element type that may legally carry `appliesWhen:`). Authoring the gate from the CLI must
be safe: an invalid expression or placement is rejected without touching the file, and a
successful set is followed by a feature-model soundness check so a gate is never applied
on top of a feature model that admits bad configurations.

### Command

- `syscribe -m <root> applies-when <element> --set "<expr>"` — set/replace the gate.
- `syscribe -m <root> applies-when <element> --clear` — remove the gate.
- `--dry-run` — show the planned change (and run the same validation) **without** writing.
- `<element>` resolves by **qualified name or stable id** (the same resolution as other
  commands; e.g. `REQ-UAV-FC-001` or `UAV::Avionics::FlightController`). An unresolved
  target **shall** exit non-zero with a clear message and write nothing.

### `--set` validation (before writing)

The command **shall** validate the new gate and **refuse to modify the file** (non-zero
exit, no write) on any of:

- **Malformed expression / unresolved operand (`E209`)** — `<expr>` is parsed with the
  same boolean grammar as the `appliesWhen:` field (`and`/`or`/`not`/parentheses; a bare
  name or list = AND). Every operand **shall** resolve to a `FeatureDef` by its qualified
  name **or** its `FEAT-*` id ([[REQ-TRS-ID-006]]); an operand that resolves to neither is
  rejected.
- **Forbidden placement (`E228`)** — the gate **shall not** be written to a `FeatureDef`,
  a `Configuration`, the model-root package, a package whose subtree contains a
  `FeatureDef`/`Configuration`, or an element on a path that already declares
  `appliesWhen:` ([[REQ-TRS-VAR-006]]).

### Writing

- On success the command **shall** write the `appliesWhen:` key into the element's YAML
  frontmatter, **preserving** every other byte of the file (other keys, body, comments,
  formatting), creating the key if absent or replacing it if present.
- `--clear` removes the key (a no-op, reported as such, if the element has none).

### Feature-model validation on set

- After a successful `--set` (and in `--dry-run`), the tool **shall** run the holistic
  **feature-model bad-configuration** checks — the `feature-check --deep` analysis: void
  model (`E223`), dead features (`E224`), and invalid configurations under full
  group/cardinality semantics (`E225`), plus the standard `requires`/`excludes` and
  configuration-satisfaction checks — over the resulting model, and report the findings.
- If the feature model admits a bad configuration (any such error), the command **shall**
  exit non-zero so the newly authored gate is not silently applied to an unsound feature
  model. `--clear` does not require this check (removing a gate cannot introduce a bad
  configuration).

**Source:** user feature request (author/maintain `appliesWhen:` from the CLI with a
feature-model safety check). Builds on [[REQ-TRS-PROJ-001]] (appliesWhen projection),
[[REQ-TRS-VAR-006]] (transitive package placement), [[REQ-TRS-ID-006]] (feature-by-id),
and the atomic file-rewrite pattern of [[REQ-TRS-MOVE-001]].

**Acceptance criteria:**

- `applies-when REQ-X --set "FEAT-ABS"` writes `appliesWhen: FEAT-ABS` into the
  Requirement's frontmatter, preserving its other fields and body; a following
  `validate`/projection treats the element as gated by that feature.
- `applies-when REQ-X --set "FEAT-NOPE"` (an operand resolving to no feature) prints an
  `E209`-style error, exits non-zero, and leaves the file unchanged.
- `applies-when <FeatureDef> --set "…"` (and the model-root package, and a package whose
  subtree contains features) is refused with an `E228`-style placement error and no write.
- After a successful `--set`, the feature-model bad-configuration check runs; a model with
  a void/dead/invalid configuration is reported and the command exits non-zero.
- `applies-when REQ-X --clear` removes the field; `--dry-run` writes nothing while still
  validating and previewing the change.
