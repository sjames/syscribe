---
type: Requirement
id: REQ-TRS-VIS-005
name: "The IBD generator derives the boundary, owned parts, ports with direction and connections from a Part or PartDef subject"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - ibd
---

For a derived `diagramKind: IBD` whose subject is a `PartDef` or `Part`, the generator shall
produce:

- a `Boundary` node for the subject carrying the subject's own ports as `Port` children on the
  boundary edge;
- one `Block` child of the boundary per owned part usage — a child `Part` element of the
  subject, or a `features:` entry typed by a `PartDef` — labelled `name : Type`, with that
  usage's ports (its own `features:` of port kind, or the ports declared on its `PartDef`) as
  `Port` children;
- for every port, the direction (`in`, `out`, `inout`) from the port usage or its `PortDef`,
  and no fixed side unless pinned;
- a `Connection`, `Flow` or `Binding` edge for each `connections:` entry on the subject and for
  each `Connection`/`Flow` element whose two ends resolve to ports or parts inside the
  boundary, using the dotted-endpoint rules `graph.rs::resolve_endpoint` implements, so that a
  connection authored by hand and one added from the diagram resolve identically.

Layout hints shall be layered, left-to-right, with hierarchy handling including children, and
port constraints fixed to a side only for ports that have one.

## Rationale

The IBD is where today's stack is weakest: ports are flattened, nesting is lost, and nothing
reads `connections:`. It is also the diagram ELK's port and hierarchy support exists for.

## Scope

- Golden IR snapshot tests on a fixture part with nested parts, boundary ports and each edge
  kind.
- A `connections:` entry whose endpoint does not resolve produces no edge and relies on the
  existing connection findings; the generator never invents a port.
