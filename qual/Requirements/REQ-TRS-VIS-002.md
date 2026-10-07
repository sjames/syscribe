---
id: REQ-TRS-VIS-002
type: Requirement
name: A single manifest parser honours the whole section 8.16 shape/edge/layout schema and reports malformed manifests as E405/W416
status: verified
reqDomain: software
verificationMethod: test
---

One parser (`vis::manifest`) **shall** build the IR from a `Diagram` element's `shapes:`,
`edges:` and `layout:` frontmatter, accepting every form spec §8.16.3–8.16.4 defines: the
string shorthand (`id: Qualified::Name`), the map form, an omitted `kind:` defaulting to
`block`, `parent:` nesting, `label:` on shapes and edges, and `layout:` entries with optional
`w`/`h`.

A `shapes:`, `edges:` or `layout:` value that is present but malformed — not a map, an entry
that is neither a string nor a map, an entry missing `ref` (shapes) or `source`/`target`
(edges), a `kind:` outside the §8.16.8 vocabulary, a `parent:` naming no shape, or a `layout:`
entry without numeric `x`/`y` — **shall** raise error `E405` naming the entry; the entry is
skipped and the rest of the diagram still builds, instead of the diagram rendering empty. A
`layout:` key that names no shape or edge of the diagram **shall** raise warning `W416`. Only a
diagram kind that has an IR is checked: a `Mermaid`/`PlantUML` body produces neither code.
`W402`/`W403` keep their meaning and are not duplicated by `E405`.

**Source:** `REQ-TRS-VIS-002` (product model).

**Acceptance criteria:** through the real walker and validator, (a) string-shorthand shapes
produce no `E405` and reach the IR; (b) a shape with `kind: gizmo` produces exactly one `E405`
naming it while the diagram's other shapes still build; (c) a stale `layout:` key produces
`W416`; (d) a `shapes:` sequence of scalars produces `E405`; (e) a `Mermaid`-kind diagram with
a malformed `shapes:` produces no `E405`. Both codes are listed in `prompts/spec/validation.md`
and `docs/validation/rules.md` (the catalogue tests enforce it).
