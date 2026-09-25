# connectivity — element-rooted subgraph of elements + connections

## SYNOPSIS
    syscribe -m <root> connectivity <element>
        [--depth N] [--format text|dot|json] [--json] [--kinds <csv>] [--undirected]

## DESCRIPTION
Walks outward from <element> over the connection/typing/containment graph and
renders the reachable elements and the connections between them. Running it on
the model-root element dumps the whole model.

## OPTIONS
    --depth N        Bound the walk to N hops (default: unbounded).
    --format <fmt>   text (indented tree, default) · dot (styled Graphviz) ·
                     json ({root, nodes, edges}).
    --json           Shorthand for --format json.
    --kinds <csv>    Edge kinds to follow (connection,flow,binding,contains,typedBy,…).
    --undirected     Follow edges in both directions.

## EXAMPLES
    syscribe -m model/ connectivity UAV::Airframe
    syscribe -m model/ connectivity UAV::Airframe --format dot | dot -Tsvg -o g.svg
    syscribe -m model/ connectivity UAV --format json --depth 2

## SEE ALSO
    tree, links, export
