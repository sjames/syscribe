---
id: REQ-TRS-VAL-016
type: Requirement
name: Tool shall make wcet queryable and flag SIL/ASIL requirements whose WCET claim has no measuring test (W029)
status: draft
reqDomain: software
verificationMethod: test
---

Worst-case-execution-time / timing bounds are central to a hard-real-time safety argument, but a `wcet:` claim is only credible if a test actually measures it. The tool **shall** make timing claims queryable and flag unbacked ones.

## Queryability

- `list <Type> --has-wcet` **shall** keep only elements that declare a non-empty `wcet:`.
- The `list --json` element object **shall** include the `wcet` value (so timing claims are machine-selectable). `wcet` is already part of the full-frontmatter `export --json`.

## `W029` — WCET claimed but not measured

The tool **shall** emit warning **`W029`** for a native `Requirement` that:

1. declares a `wcet:` value, **and**
2. carries an integrity level (`silLevel` or `asilLevel`), **and**
3. is **not** `status: draft`, **and**
4. has **no active "measuring" TestCase** verifying it — where a *measuring* TestCase is one (with `status: active`, in this requirement's `verified_by` set) whose `testLevel` is `L5` (HIL/physical) **or** whose `tags:` include `timing` or `wcet`.

`W029` **shall** be opt-in (it cannot fire unless a `wcet:` claim and an integrity level are both present), draft-suppressed, and gateable via `validate --deny W029`. It is the timing-evidence analog of `W702` (ASIL-D requires an L5 test).

**Source:** GH #22; complements the list/matrix query options of [[REQ-TRS-DISC-007]] and the SIL/ASIL verification-rigor rules of [[REQ-TRS-VAL-008]].

**Acceptance criteria:** `list Requirement --has-wcet` returns only requirements with a `wcet:`; `list --json` includes `wcet`. A non-draft SIL/ASIL requirement with `wcet:` and no measuring (L5 or timing/wcet-tagged active) TestCase produces exactly one `W029`; adding such a TestCase clears it; a requirement without `wcet:` or without an integrity level produces none; `validate --deny W029` exits non-zero when a `W029` is present.
