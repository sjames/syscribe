---
id: REQ-TRS-FMEA-002
type: Requirement
name: "FMEA entry canonical field vocabulary, RPN auto-compute, and unknown-key error (E922)"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce a single, consistent YAML field vocabulary for FMEA entry rows
in `entries:` lists.

## Canonical field names

The canonical key for entry severity is **`fmeaSeverity:`** (integer 1–10). The deprecated
alias **`severity:`** is accepted and silently mapped to `fmeaSeverity` to support existing
models authored against earlier templates. The deprecated alias **shall not** raise a
diagnostic; fixing it is a future tightening.

The complete set of recognised FMEA entry keys is:

`id`, `ref`, `name`, `failureMode`, `status`, `effect`, `cause`,
`fmeaSeverity`, `severity`, `occurrence`, `detection`, `rpn`,
`recommendedAction`, `satisfies`.

## E922 — unknown FMEA entry key

Any key in an `entries:` row that is **not** in the recognised set **shall** raise error
**`E922`** with a message naming the unknown key. Silent drops in a safety analysis are not
acceptable.

## RPN auto-compute

When `rpn:` is absent from an entry row but all three of `fmeaSeverity:` (or `severity:`),
`occurrence:`, and `detection:` are present, the tool **shall** automatically compute
`RPN = S × O × D` and apply that computed value to the W903 threshold check. Explicitly
supplying `rpn:` overrides the computed value.

**Acceptance criteria:**

- An FMEASheet entry with `fmeaSeverity: 9, occurrence: 9, detection: 9` (no explicit
  `rpn:`) and no `recommendedAction:` raises **W903** with a message containing the
  computed RPN (729).
- An entry with `severity: 9` (deprecated alias) and the same conditions also raises
  **W903** — the alias is accepted.
- An entry with an unrecognised key (e.g. `failureEffect:`) raises **E922** naming that
  key.
- `syscribe template FMEASheet` emits `fmeaSeverity:` (not `severity:`).
