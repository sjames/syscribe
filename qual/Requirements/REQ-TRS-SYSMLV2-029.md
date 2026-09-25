---
id: REQ-TRS-SYSMLV2-029
type: Requirement
name: Tool shall map a SysMLv2 allocation def to a native AllocationDef and check an ingested allocation usage's typedBy like any other (E111)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** synthesize a SysMLv2 `allocation def` — at package level or nested in a
`part def` body — into a native **`AllocationDef`** element carrying `supertype:` (its `:>`
clause) and its `doc` comment. An ingested `allocation` usage's `typedBy:` **shall** then be
resolved like any other `typedBy:` reference: one that names an in-model `AllocationDef`
resolves; one that resolves nowhere raises **`E111`**. The former blanket exemption of ingested
`Allocation` elements from `E111` (which also hid genuinely dangling references) **shall** no
longer apply.

**Source:** `REQ-TRS-SYSMLV2-029` (product model), GH #142.

**Acceptance criteria:**

- A package-level `allocation def` and one nested in a `part def` each appear as an
  `AllocationDef` under their derived qualified names; the doc comment is lifted.
- `allocation a : SomeAllocDef;` whose type is an ingested `allocation def` raises no `E111`.
- `allocation a : NoSuchDef;` raises `E111` naming `NoSuchDef`.
