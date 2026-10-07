---
id: REQ-TRS-VIS-009
type: Requirement
name: PlantUML and Mermaid are written from the Diagram IR and reachable through diagram export and MCP render_diagram
status: verified
reqDomain: software
verificationMethod: test
---

The Mermaid writer (`vis::mermaid`) **shall** be a pure function of the Diagram IR, mapping a
BDD to a `classDiagram` (compartment lines as members, `<<stereotype>>` annotations, `<|--`
inheritance, `*--` composition with the usage label, `--` association, `..>` dependency) and an
IBD to a `flowchart` with a `subgraph` per boundary and nested block and ports as small nodes,
and **shall** precede every node line with `%% ref: <QualifiedName>` so the existing
`W408`/`W409` lints apply to generated text exactly as to a hand-written block. The PlantUML
writer keeps reading the same IR (`REQ-TRS-VIS-001`).

`syscribe diagram export <qname> --format plantuml|mermaid|svg [--out <file>]` **shall** write
the chosen output for one `Diagram` element to stdout, or to `--out` (creating parent
directories); `export` is the only `diagram` subcommand. An unresolvable `<qname>` **shall**
print `error: element '<qname>' not found` on stderr, write nothing to stdout and exit `1`
(the not-found rule of the retired `REQ-TRS-DIAG-004`); a `<qname>` that is not a `Diagram`
**shall** exit `1` with `error: '<qname>' is not a Diagram`; a `--format` outside the three
values **shall** be a usage error naming the valid values, exit `1`. MCP `render_diagram`
**shall** accept `format: mermaid` (and `svg`) alongside the default `plantuml`, returning the
same text as the CLI. When `[links]` is configured, Mermaid output **shall** carry a
`click <id> href "<url>" _blank` line per linked node.

**Source:** `REQ-TRS-VIS-009` (product model).

**Acceptance criteria:** (a) `diagram export Nope::X` exits 1 with the not-found message and
empty stdout; (b) `--format mermaid` of a manifest BDD prints a `classDiagram` with a
`%% ref:` per node; (c) `--out` writes the file; (d) `--format svg` of an unpinned diagram
is refused (`REQ-TRS-VIS-010`); (e) `--format png` is a usage error naming
`plantuml, mermaid, svg`; (f) MCP `render_diagram {format: mermaid}` returns the generated
Mermaid; (g) the Mermaid and PlantUML text for the derived BDD/IBD fixture match their
golden snapshots.
