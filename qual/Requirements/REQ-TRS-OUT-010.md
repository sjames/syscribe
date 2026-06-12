---
id: REQ-TRS-OUT-010
type: Requirement
name: Tool shall export the transitive connectivity subgraph rooted at an element (text, JSON, DOT)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a `connectivity` command that exports the *connected slice* of the model reachable from a chosen root element — the elements plus the connections between them — so a focused subsystem (or the whole model) can be fed as context or reasoned about. Running it on the model-root element **shall** dump the whole model.

**Part A — connection wiring in the model graph.** The graph builder (`build_graph`) **shall** resolve the port-to-port wiring declared on `Part`/`PartDef`/`Action`/`ActionDef` into traversable edges. Four edge kinds **shall** be populated from frontmatter:

- `Connection` — from `connections:` entries (binary `from`/`to`, or n-ary `ends: [{end, binds}]`);
- `Flow` — from `flowConnections:` entries (`from`/`to`);
- `Binding` — from `bindingConnections:` entries (`left`/`right`);
- `Succession` — from `successionConnections:` entries (`from`/`to`).

Each endpoint is a feature chain `c`. Resolution **shall** be: let `head` be the segment of `c` before the first `.`; if the owning element's `features:` declares an entry whose `name == head` carrying `typedBy: T`, the endpoint node is `resolve_ref(T)`; otherwise `resolve_ref(c)` (the whole chain as a qname/id); otherwise the endpoint is unresolved and **shall** be skipped. A binary connection adds one edge between its two resolved endpoints; an n-ary connection adds a star from the first resolved endpoint to each other resolved endpoint. Self-edges (`head` resolving to the owner) may be skipped.

To let a structural walk reach a part's sub-part *types* (so the connection edge between them is reachable from the owning part), the builder **shall** additionally emit a `FeatureTyped` edge from each element to the type of every inline `features:` entry that declares `typedBy:`. This edge is deliberately **distinct** from the top-level `TypedBy` kind so it does **not** participate in `typedBy` cycle detection (E107). All of these additions **shall** be additive — existing cycle detection and server consumers **shall** be unaffected.

**Part B — `connectivity` command.**

```
syscribe -m <root> connectivity <element> [--depth N] [--format text|dot|json] [--kinds <csv>] [--undirected]
```

The command **shall**:

- resolve `<element>` (qname or stable id) via the resolver; an unknown root **shall** print to stderr and exit non-zero;
- traverse the graph outward from the root, following a default set of edge kinds — `Connection,Flow,Binding,Succession,FeatureTyped,Contains,TypedBy` — so the model-root element dumps the whole model via `Contains` and a part reaches its sub-part types via `FeatureTyped`; `--kinds <csv>` (case-insensitive kind names) **shall** override the default set; `--depth N` **shall** bound the number of hops (default unbounded); `--undirected` **shall** follow edges in both directions (default: outbound only);
- collect the reachable nodes and the deduplicated edges among them;
- render in three formats selected by `--format` (default `text`):
  - **text** — an indented tree from the root using `├──`/`└──`/`│  ` connectors; each node line is `<qualifiedName> [<Type>]`; each child line is prefixed with its edge kind, e.g. `└── [connection] PortDemo::Motor [PartDef]`; an already-expanded node is shown but not re-expanded (marked to avoid infinite loops);
  - **json** — `{ "root": "<qname>", "nodes": [{"qualifiedName","type","id"}], "edges": [{"from","to","kind"}] }`;
  - **dot** — Graphviz DOT styled by a single source-of-truth node-style function: shape by element family, pale fill with a saturated border by domain, `peripheries=2` for definition types, an edge style per kind, and a legend subgraph.

**Source:** Issue #26 (connectivity export — element-rooted transitive subgraph).

**Acceptance criteria:** Given a parent `PartDef` with two sub-part features wired by a `connections:` entry, `connectivity <parent>` text output names both sub-parts; `--format json` is valid JSON exposing `nodes` and `edges`, including a `connection`-kind edge between the two sub-parts; `--format dot` contains `digraph` and a shape/`peripheries` attribute; `connectivity` on the model-root element reaches every element; `--depth 0`/`1` bounds the walk; an unknown root exits non-zero.
