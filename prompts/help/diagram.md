# diagram — export one Diagram element as PlantUML, Mermaid or static SVG

## SYNOPSIS
    syscribe -m <root> diagram export <qname> [--format plantuml|mermaid|svg|dot] [--config <C>] [--out <file>]
    syscribe -m <root> diagram export --all [--format svg|mermaid|dot|plantuml] [--config <C>] [--out-dir <dir>] [--check]

## DESCRIPTION
Writes one `Diagram` element's picture source, generated from its Diagram IR
(the `shapes:`/`edges:`/`layout:` manifest, or the content derived from its
`subject:`), to stdout or to `--out`. `export` is the only `diagram`
subcommand; the former toolkit (`list`, `render`, `measure`, `compose`,
`layout`, `seq`, `req`) was removed under `ADR-SYS-VIS-001`; automatic layout
is ELK, in the browser and embedded in this executable alike.

    plantuml   PlantUML source (the default) — the same text `syscribe plantuml`
               writes to a companion `.puml`, with `[plantuml] base_url` links.
    mermaid    Mermaid text: BDD and Requirement diagrams become a
               `classDiagram`, IBD/Allocation/UseCase/Custom a `flowchart` with
               a `subgraph` per boundary and nested block, StateMachine a
               `stateDiagram-v2`, Sequence a `sequenceDiagram`. Every node is
               preceded by `%% ref: <QualifiedName>`, so the `W408`/`W409`
               lints apply to the generated text exactly as to a hand-written
               block.
    dot        Graphviz DOT (a node per shape coloured by its tone, an edge per
               edge), written for the FaultTree / AttackTree / SafetyCase
               diagrams (`diagramKind:` of the safety analyses, derived from
               their `subject:`) and the Traceability / ZoneConduit /
               ThreatGraph analysis graphs (zones are clusters) but total
               over every kind.
    svg        A standalone SVG per spec §8.16.5 (`sysml:ref` on every shape,
               `sysml:ref`/`sysml:source`/`sysml:target` on every edge) for
               any diagram with an IR. A fully pinned diagram is drawn from
               its `layout:` pins; any other is laid out first by the embedded
               ELK — the browser's own `elk.bundled.js`, run in-process with
               the same options and the same Rust-computed node sizes, so the
               picture matches the browser's — with pins still honoured. No
               Node, browser or network is involved.

When `.syscribe.toml` has a `[links]` table, Mermaid output gains a
`click <id> href "<url>" _blank` line per linked node and every linked SVG shape
is wrapped in `<a xlink:href="<url>" href="<url>" target="_blank" rel="noopener">`
(REQ-TRS-LINK-002); without `[links]` both are inert.

`--all` (GH #339) writes every `Diagram` to `<out-dir>/<qname path>.<ext>`
(default `<root>/generated`, or `<root>/generated/<CONF-id>` with `--config`;
SVG unless `--format` says otherwise) and records what it wrote in
`<out-dir>/.syscribe-generated`. It rewrites only files whose content changed
and removes only a file that record names and no diagram renders any more; a
directory containing the model is refused. A hand-authored diagram, a kind
with no mapping in the format and a companion picture are reported `skipped:`;
any other render failure is an error that keeps the diagram's file and exits 1.
`--check` writes nothing and exits 1 naming each file that differs, is missing
or is left over, so a committed directory of renders can gate CI. Text is
measured with metrics compiled into syscribe, never the installed fonts, and a
PlantUML style `!include` is written relative to the file, so a render is the
same bytes on every machine and checkout.

## OPTIONS
    --format <plantuml|mermaid|svg|dot>   Output format (default: plantuml). Any
                                      other value is a usage error.
    --config <C>                      Derive from the model projected onto Configuration <C>
                                      (elements gated off by appliesWhen are absent). An
                                      unknown configuration, or a diagram inactive in it,
                                      exits 1.
    --out <file>                      Write to <file> (parent directories are
                                      created) instead of stdout.
    --all                             Every Diagram of the model, into --out-dir.
    --out-dir <dir>                   With --all: the directory (default <root>/generated,
                                      or <root>/generated/<CONF-id> with --config).
    --check                           With --all: write nothing; exit 1 when a file is
                                      out of date, missing or left over.

## EXIT CODES
    0  written
    1  element '<qname>' not found (nothing on stdout); '<qname>' is not a
       Diagram; svg requested for a diagram with no shapes to draw; a
       diagramKind with no mapping for the format; invalid --format; with
       --all, a diagram that fails to render, an --out-dir containing the
       model, or (with --check) a render out of date

## EXAMPLES
    syscribe -m model/ diagram export Diagrams::PowerSystemDerivedIBD --format mermaid
    syscribe -m model/ diagram export Diagrams::SafetyRequirementsD --format svg --out site/SafetyRequirementsD.svg
    syscribe -m model/ diagram export Diagrams::UAVSystemBDD > UAVSystemBDD.puml
    syscribe -m model_auto/ diagram export Diagrams::ZoneConduitEngine --format dot
    syscribe -m model_auto/ diagram export Diagrams::TraceabilityEngine --format mermaid
    syscribe -m model_auto/ diagram export Diagrams::ThreatGraphEngine --format svg --out threats.svg
    syscribe -m model_gti/ diagram export --all --check

## NOTES
The `Traceability`, `ZoneConduit` and `ThreatGraph` kinds (GH #223) are derived from
a `subject:` like the safety kinds: a hazard-to-test graph (HazardousEvent, SafetyGoal,
Requirements, TestCases), the IEC 62443 zones (compound nodes) and conduits (edges), and
the TARA chain (threat, damage, asset, goal, control). The threat graph's risk colours
follow the `[cyber]` method of `.syscribe.toml`.

## SEE ALSO
    plantuml, render, export-html, zones, hara, cyber-risk
