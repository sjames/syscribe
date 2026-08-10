---
type: Requirement
id: REQ-TRS-SYSMLV2-011
name: "n2's subpart axis shall include a scope's synthesized SysMLv2 children via containment, not only features:-declared subparts"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - connectivity
---

`n2 <qname>`'s subpart-axis selection shall include, alongside its existing `features:`-declared
subparts, every `PartDef`/`Part` whose qualified name is a direct child of the scope element (i.e.
`<scope qname>::<name>`, no further `::` segments) — so a `sysmlSubmodel: true` subtree's
SysMLv2-synthesized children (`REQ-TRS-SYSMLV2-002`'s existing qname-containment mapping, never
expressed via `features:`) populate `n2`'s axis, and `REQ-TRS-SYSMLV2-010`'s lifted connection
edges between them populate the corresponding off-diagonal cells.

## Rationale

`n2` is the standard systems-engineering N² interface-matrix artifact this whole graph exists to
feed, and until this requirement it cannot see a `sysmlSubmodel: true` subtree at all: `n2`'s
subpart-axis selection (`crates/syscribe/src/n2.rs::subpart_axis`) walks only a `features:` YAML
list, the native-Markdown convention for declaring inline-typed subparts — but a SysMLv2 element's
subparts are separate, qname-nested `RawElement`s (`REQ-TRS-SYSMLV2-002`), never `features:`
entries. `n2 <sysmlv2-subtree-root>` reports `(no parts in scope)` regardless of how much real
`connection` wiring the subtree contains (confirmed even after `REQ-TRS-SYSMLV2-010`'s lift
produces genuinely resolvable `connectivity` edges for the same subtree).

## Scope

- Widens `subpart_axis` (used by **scoped** `n2 <qname>`) with a second, additive source of
  subpart membership: direct-child containment by qname. The existing `features:`-based source is
  unchanged and still contributes (a hand-authored model mixing both conventions gets the union,
  de-duplicated).
- **Unscoped `n2`** (no `<qname>` argument, whole-model axis) already includes every
  `PartDef`/`Part` in the model regardless of origin — confirmed unaffected by this gap and
  unchanged by this requirement. What was actually missing there was the ordinary
  `connectivity`/`n2` outcome for an edge whose lifted endpoint resolves to a non-`Part`-typed
  element (a `Port`, for instance) — `n2`'s axis is deliberately `Part`/`PartDef`-only
  (`is_part`), unchanged by this requirement; that class of edge simply never appears in `n2`,
  same as it already didn't for a hand-authored model with the same shape. Not a regression this
  requirement introduces.
- Does not widen `n2`'s axis to non-`Part`/`PartDef` element kinds (`Port`, `Interface`, etc.) —
  `is_part`'s existing filter is unchanged.
- Does not resolve dotted (`a.x`) connection endpoints to a finer-than-part granularity — that is
  a separate, explicitly out-of-scope concern (tracked separately); a bare part-to-part edge is
  sufficient to populate a cell.
- A model with no `sysmlSubmodel: true` package, and no nested-file `PartDef`/`Part` containment
  under a scope either, is unaffected — the added containment source contributes nothing new,
  `features:`-only resolution behaves exactly as before, no regression.

**Acceptance criteria:** `n2 <sysmlv2-subtree-root>` lists every direct-child `Part` of that root
on the diagonal; a `REQ-TRS-SYSMLV2-010`-lifted connection between two such parts populates the
corresponding off-diagonal cell; a hand-authored model using only `features:` (no qname-nested
children) behaves exactly as it does today; a hand-authored model that happens to nest a
`PartDef`/`Part` as a real child file under a composite gains the same axis inclusion (a strict
widening, not a SysMLv2-only special case).
