---
id: REQ-TRS-VIS-012
type: Requirement
name: One visual language, owned in Rust, is resolved per node, port and edge and carried in the sprotty model so the client's views hold no table of their own
status: verified
reqDomain: software
verificationMethod: test
---

`vis::style` **shall** define the visual language once. For a node it **shall** resolve fill,
stroke, optional header fill, text colour and a dashed flag by the element type first (`PartDef`
and `Part` alike; `Requirement` and `TestCase` with a header fill; `PortDef`, `Interface`,
`ConnectionDef`, `ActionDef`, `State`, `UseCaseDef`, `Allocation` each their own colours), then by
the node kind when the type decides nothing (a `Package` drawn as a boundary falls back to the
boundary colours; note, actor, lifeline, block, fragment, swimlane and system boundary each have
theirs), and **shall** dash an unresolved node. For an edge it **shall** follow spec §8.16.8's
tables row by row — stroke, dash pattern, target and source arrowhead and keyword — for all
twenty-four edge kinds: none for connection, a filled arrow for flow, dashed with `=` for binding,
a hollow triangle for inheritance, a filled diamond at the source for composition, a hollow
diamond for aggregation, an open arrow with the `«keyword»` for the traceability kinds, and one
shared width. For a port it **shall** resolve fill, stroke and the glyph `in`, `out`, `inout` or
`none` by direction. Every style **shall** serialise camelCase with explicit `null` for an absent
header fill, dash or keyword, and the sprotty model **shall** carry the resolved style per node,
port and edge, plus a `«Name»` banner per applied `MetadataDef` stereotype and a port `side`, so
the client's views read them rather than hold their own table.

The SVG writer's use of the same table is Phase 3 and is verified under `REQ-TRS-VIS-010`.

**Source:** `REQ-TRS-VIS-012` (product model).

**Acceptance criteria:** (a) `node_style` returns the expected colours for each typed element,
the kind fallback for a `Package` boundary and every untyped kind, and `dashed` for an
unresolved node; (b) `edge_style` matches the spec table for all twenty-four kinds, every row
checked; (c) `port_style` yields the four glyphs with their colours; (d) the serialised JSON spells
`arrowSource: filledDiamond`, `arrowTarget: hollowTriangle`, and `null` for an absent
`headerFill`/`dash`/`keyword` (`TC-TRS-VIS-012`); (e) the sprotty writer's
`nodes_ports_and_edges_carry_their_resolved_style` (`vis/sprotty.rs`) and the served IBD of
`TC-TRS-VIS-006` show the same values on every node, port and edge of a real graph.
