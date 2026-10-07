# diagram — export one Diagram element as PlantUML, Mermaid or static SVG

## SYNOPSIS
    syscribe -m <root> diagram export <qname> [--format plantuml|mermaid|svg] [--out <file>]

## DESCRIPTION
Writes one `Diagram` element's picture source, generated from its Diagram IR
(the `shapes:`/`edges:`/`layout:` manifest, or the content derived from its
`subject:`), to stdout or to `--out`. `export` is the only `diagram`
subcommand; the former toolkit (`list`, `render`, `measure`, `compose`,
`layout`, `seq`, `req`) was removed under `ADR-SYS-VIS-001` and layout now
happens in the browser.

    plantuml   PlantUML source (the default) — the same text `syscribe plantuml`
               writes to a companion `.puml`, with `[plantuml] base_url` links.
    mermaid    Mermaid text: BDD and Requirement diagrams become a
               `classDiagram`, IBD/Allocation/UseCase/Custom a `flowchart` with
               a `subgraph` per boundary and nested block, StateMachine a
               `stateDiagram-v2`, Sequence a `sequenceDiagram`. Every node is
               preceded by `%% ref: <QualifiedName>`, so the `W408`/`W409`
               lints apply to the generated text exactly as to a hand-written
               block.
    svg        A standalone SVG per spec §8.16.5 (`sysml:ref` on every shape,
               `sysml:ref`/`sysml:source`/`sysml:target` on every edge), drawn
               only from pins: every node must carry a `layout:` entry (open
               the diagram in the browser and use Pin all). The server never
               computes a layout.

When `.syscribe.toml` has a `[links]` table, Mermaid output gains a
`click <id> href "<url>" _blank` line per linked node and every linked SVG shape
is wrapped in `<a xlink:href="<url>" href="<url>" target="_blank" rel="noopener">`
(REQ-TRS-LINK-002); without `[links]` both are inert.

## OPTIONS
    --format <plantuml|mermaid|svg>   Output format (default: plantuml). Any
                                      other value is a usage error.
    --out <file>                      Write to <file> (parent directories are
                                      created) instead of stdout.

## EXIT CODES
    0  written
    1  element '<qname>' not found (nothing on stdout); '<qname>' is not a
       Diagram; svg requested for a diagram that is not fully pinned; a
       diagramKind with no mapping for the format; invalid --format

## EXAMPLES
    syscribe -m model/ diagram export Diagrams::PowerSystemDerivedIBD --format mermaid
    syscribe -m model/ diagram export Diagrams::SafetyRequirementsD --format svg --out site/SafetyRequirementsD.svg
    syscribe -m model/ diagram export Diagrams::UAVSystemBDD > UAVSystemBDD.puml

## SEE ALSO
    plantuml, render, export-html
