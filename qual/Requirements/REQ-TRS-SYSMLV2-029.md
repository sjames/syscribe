---
id: REQ-TRS-SYSMLV2-029
type: Requirement
name: Tool shall map a SysMLv2 allocation def to a native AllocationDef and check an ingested allocation usage's typedBy like any other (E111), and shall lift its allocate clause into allocatedFrom/allocatedTo
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

A named `allocation` usage's `allocate <source> to <target>` clause **shall** be lifted onto the
synthesized `Allocation` as `allocatedFrom: [<source>]` / `allocatedTo: [<target>]`, so the
ingested allocation contributes an edge to the §12.9 unified allocation set (`E314`, `W034`,
`matrix --allocations`) exactly like a hand-authored `type: Allocation` element. Each endpoint
**shall** be resolved against the full model at ingest time: a (possibly `::`-qualified) name
resolves innermost-scope-first from the allocation's owning namespace outward to the model root;
each further `.`-chain segment resolves as a feature of the element reached so far — declared
directly on it, or inherited through its `typedBy:`/`supertype:` chain. A chain whose tail cannot be
resolved **shall** be truncated to its deepest resolved prefix with a `W542` warning; an endpoint
whose head resolves nowhere is kept verbatim (`.` rewritten to `::`) and reported by the existing
`E502`/`E503` checks. A stable id (`REQ-*`, …) is kept as-is.

**Source:** `REQ-TRS-SYSMLV2-029` (product model), GH #142, GH #144.

**Acceptance criteria:**

- A package-level `allocation def` and one nested in a `part def` each appear as an
  `AllocationDef` under their derived qualified names; the doc comment is lifted.
- `allocation a : SomeAllocDef;` whose type is an ingested `allocation def` raises no `E111`.
- `allocation a : NoSuchDef;` raises `E111` naming `NoSuchDef`.
- `allocation deploy allocate Logical::ctrl to Physical::ecu;` shows `allocatedFrom`/`allocatedTo`
  resolved to the full qualified names, and `matrix --allocations` lists the edge.
- An `isDeploymentPackage: true` part allocated to a `hardware` part only through an ingested
  allocation raises no `E314`.
- `allocate sys.ctl to board.mcu` resolves `ctl`/`mcu` through the heads' part-def types; a chain
  naming a feature that exists nowhere is truncated to its head with `W542`; an endpoint naming
  nothing raises `E502`/`E503`.
