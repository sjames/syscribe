---
id: REQ-TRS-QUAL-001
type: Requirement
name: The qualification model shall validate with no errors and no unrecognised frontmatter fields, and the TVR shall take its tool version from the qualified binary
status: draft
reqDomain: software
verificationMethod: test
---

The qualification model (`qual/` — this Tool Requirements Specification, its test
cases and its package `_index.md`) is itself a Syscribe model and **shall** validate
with the qualified binary (`syscribe -m qual validate`) with **zero errors** and **no
`W047`** (unrecognised frontmatter field) finding on any of its files. Author-defined
data that the schema does not define **shall** live under `custom_fields:` (§3.15),
never as a bare top-level key.

The Tool Validation Report (`qual/tests/tvr/TVR.md`) **shall** record the tool version
reported by the qualified binary's own `--version` (passed by `run_qual.sh` to
`generate_tvr.sh`), never a version string maintained by hand in model data — so the
report cannot drift from the binary it qualifies.

**Source:** GH #173 — `qual/_index.md` carried a `version: "0.1"` key that no consumer
read (the TVR already used the binary's `--version`) and that raised `W047`.

**Acceptance criteria:**

- `syscribe -m qual validate` exits 0 and reports no `W047` row.
- `qual/_index.md` carries no top-level `version:` key.
