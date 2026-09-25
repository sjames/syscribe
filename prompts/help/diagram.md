# diagram — diagram authoring workflow

## SYNOPSIS
    syscribe -m <root> diagram list [--type <T,...>] [--namespace <prefix>]
    syscribe -m <root> diagram measure <qname>[,<qname>...] [<view options>]
    syscribe -m <root> diagram render <qname> [<view options>] [--output|-o <file>]
    syscribe -m <root> diagram compose <layout.json|Diagram-qname> [--output <file>]
        [--kind bdd|ibd|arch] [--emit-placement]
    syscribe -m <root> diagram layout <placement.json|-> [--output <file>]
        [--compose [--kind bdd|ibd|arch] [--svg <file>]]
    syscribe -m <root> diagram seq <Diagram-qname> [--output <file>]
    syscribe -m <root> diagram req <REQ-id|qname> [--depth <N>]
        [--show-verify] [--show-satisfy] [--output <file>]

## DESCRIPTION
Generates SysML block, sequence and requirement diagrams as SVG, and drives the
companion-SVG authoring workflow (measure → place → layout → compose). Every
subcommand writes to stdout unless `--output` is given. `diagram <sub> --help`
shows this page.

    list      Every model element as JSON (qname, type, …) — the candidates for a
              diagram. --type filters by element type(s), comma-separated
              (e.g. PartDef,Part); --namespace by qualified-name prefix.
    measure   Computed box sizes and port anchors, as JSON, for a comma-separated
              list of qualified names.
    render    One element as a standalone SVG block.
    compose   Assemble elements + edges into a full diagram SVG, from a
              *.layout.json file or from a View/Diagram element that has an
              `expose:` list. --kind picks the diagram kind (default arch);
              --emit-placement writes the auto-generated placement JSON instead of
              SVG, for hand/LLM refinement and a later `diagram layout`.
    layout    Solve element positions from a col/row placement file (or `-` for
              stdin) with Cassowary constraints; prints the resolved layout JSON.
              --compose pipes the result straight into compose and emits SVG
              (--svg sets that SVG's path; --kind the diagram kind, default: from
              the placement file).
    seq       Render a Diagram element with `diagramKind: Sequence` as SVG.
    req       Auto-laid-out requirement-breakdown tree rooted at a requirement
              (derive edges); --depth bounds it, --show-verify adds TestCases
              («verify»), --show-satisfy adds satisfying architecture («satisfy»).

An element that does not exist is an error, never a warning: when a `measure`
qname, a placed qname in a layout/placement file, an `expose:` entry, or a
`render`/`seq`/`req` target does not resolve, the command prints
`error: element '<qname>' not found` on stderr for each, writes nothing to
stdout, and exits 1.

View options (measure, render) — what each box shows:

    --view <preset>            full (default) | ports | features | compact | name |
                               requirement (alias req)
    --include-ports <csv>      only these port names
    --include-features <csv>   only these feature names
    --min-width <N>            minimum box width in pixels

Layout and placement files are ephemeral inputs and must not be committed — name
them `*.layout.json` so `.gitignore` excludes them.

## EXAMPLES
    # against the bundled model/ demo
    syscribe -m model/ diagram list --type PartDef
    syscribe -m model/ diagram measure UAV::Avionics::FlightController,UAV::Power::PowerSystem
    syscribe -m model/ diagram render UAV::Avionics::FlightController --output fc.svg
    syscribe -m model/ diagram render UAV::Avionics::FlightController --view ports --min-width 240
    syscribe -m model/ diagram compose Views::SystemArchitectureView --output arch.svg
    syscribe -m model/ diagram compose Views::SystemArchitectureView --emit-placement \
      | syscribe -m model/ diagram layout - --compose --svg arch.svg
    syscribe -m model/ diagram seq Diagrams::MissionExecutionSeq --output mission.svg
    syscribe -m model/ diagram req REQ-UAV-SAFE-001 --depth 2 --show-verify --show-satisfy

## EXIT CODES
    0  success    1  unknown element / missing `expose:` / unreadable input    2  invalid arguments (clap)

## SEE ALSO
    render, plantuml
