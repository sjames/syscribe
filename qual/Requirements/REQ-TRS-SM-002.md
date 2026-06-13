---
id: REQ-TRS-SM-002
type: Requirement
name: Tool shall deprecate the legacy from/to/trigger transition keys (W075) while accepting them as aliases
status: draft
reqDomain: software
verificationMethod: test
---

The legacy top-level transition spelling `from:` / `to:` / `trigger:` is **not** SysMLv2
vocabulary (§7.18.3 uses `source` / `target` / `accept`). The tool **shall** retire it onto
the canonical schema of [[REQ-TRS-SM-001]] without breaking existing models:

- The tool **shall** continue to accept the legacy keys as **aliases** so existing models
  keep parsing: `from:` ≡ `source:`, `to:` ≡ `target:`, `trigger:` ≡ `accept.payload`. A
  legacy transition contributes the same `(source → target)` edge as its canonical form.
- A `StateDef` / `State` transition that uses **any** of `from:`, `to:`, or `trigger:`
  **shall** raise warning **`W075`**, naming the element and directing the author to the
  canonical `source` / `target` / `accept` keys.
- `W075` **shall** be **draft-suppressed** (not emitted for elements with `status: draft`)
  and **gateable** with `--deny W075`, consistent with the other state-machine warnings.

The tool's own demonstration models **shall** carry no `W075` findings: the legacy usage in
`model_mg` (the only StateDef using `from`/`to`/`trigger`) **shall** be migrated to the
canonical schema.

**Source:** schema consolidation onto the canonical SysMLv2 transition vocabulary
([[REQ-TRS-SM-001]]); GH #68 follow-up.

**Acceptance criteria:**

- A `StateDef` whose transition uses `from:`/`to:`/`trigger:` raises `W075` and still
  contributes the correct edge.
- The equivalent transition written with `source:`/`target:`/`accept:` raises **no** `W075`.
- `W075` is suppressed when the element is `status: draft`.
- `validate --deny W075` over a model containing a legacy transition exits non-zero.
- The shipped models (`model`, `model_mg`, `model_sil`, `model_auto`) are `W075`-clean.
