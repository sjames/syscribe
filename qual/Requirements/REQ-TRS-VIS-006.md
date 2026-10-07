---
id: REQ-TRS-VIS-006
type: Requirement
name: The diagram-model endpoint serves a nested sprotty graph with ports, labels, compartments, resolved styles, ELK layout options and the pinned set; PATCH writes or removes one pin and DELETE removes every pin
status: verified
reqDomain: software
verificationMethod: test
---

`GET /api/diagrams/model/{qname}` **shall** serialise a `Diagram` element's IR — built from its
`shapes:`/`edges:`/`layout:` manifest, or derived from its `subject:` when it declares no
`shapes:` — as a **nested** sprotty `SGraph`: a `node` per IR node placed inside its parent's
`children`, `port` children for ports (carrying `direction` and a `side`), a `label` child for
the display name, a `compartment` child for compartment lines, and an `edge` per IR edge at the
**root** carrying its `kind` and any pinned `routingPoints`. `position` **shall** be present only
for pinned nodes and `size` only when the pin carries `w`/`h`. The root **shall** carry a
`layoutOptions` map of ELK option ids derived from the IR's layout hints for the diagram's kind
(layered left-to-right with `INCLUDE_CHILDREN` and `FIXED_SIDE` for an IBD; top-down with
`inheritance` in `syscribe.reversedEdgeKinds` for a BDD) and a `pinned` list of node ids. Every
node, port and edge **shall** carry its resolved `style` (`REQ-TRS-VIS-012`). An unresolved ref
**shall** be served (dashed), never dropped. The qname **shall** be accepted with either `::` or
`/` separators; a Mermaid-kind diagram, a non-diagram element and an unknown name **shall** be
`404`.

`PATCH /api/diagrams/layout/{qname}` **shall** keep its shape and write each entry as a pin; an
entry whose value is `null` **shall** remove that pin and leave the others. `DELETE
/api/diagrams/layout/{qname}` **shall** remove every pin of the diagram. Both are guarded writes.

**Source:** `REQ-TRS-VIS-006` (product model).

**Acceptance criteria:** through the real router built as `main` builds it, (a) an IBD fixture is
served with its port nested in its block and its block in the boundary, edges only at the root,
`position`/`size` only on the one pinned node and `pinned` naming it, the IBD `layoutOptions`,
and an unresolved shape drawn dashed; (b) a BDD fixture written in shorthand and map form carries
`elk.direction: DOWN`, no hierarchy handling, `inheritance` reversed and an empty `pinned`;
(c) `Diagrams::X` and `Diagrams/X` return identical JSON; (d) a `subject:`-only diagram returns
the derived graph; (e) a Mermaid diagram, a `PartDef` and an unknown name are `404`
(`TC-TRS-VIS-006`). The `PATCH`-`null` and `DELETE` behaviour of the second paragraph is verified
by `TC-TRS-VIS-011`, which also lists this requirement.
