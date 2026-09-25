# impact — change impact analysis over the traceability graph

## SYNOPSIS
    syscribe -m <root> impact <qname|id> [--direction downstream|upstream|both]
                               [--depth <N>] [--format text|json|dot] [--kinds <csv>]

## DESCRIPTION
Traverses the traceability graph (§17) from a named element and reports every reachable
node, its hop distance, and the edge kind that connects it — "if I change this, what else
may need to change?". Read-only.

**Downstream** follows reverse links (who depends on me): `specializedBy`, `derivedChildren`,
`verifiedBy`, `satisfiedBy`, `refinedBy`, `conditionalOn`, `allocatedFrom`,
`safetyGoalChildren`. **Upstream** follows forward links (what I depend on): `supertype`,
`derivedFrom`, `verifies`, `satisfies`, `refines`, `allocatedTo`, `derivedFromSafetyGoal`.

User-defined links (`links:`, declared in `[linkTypes]` of `.syscribe.toml`) are traversed
too: upstream along the link (source → target, labelled with the type name), downstream
against it (labelled with the type's `inverse`, or `<type> (inbound)`).

## OPTIONS
    --direction D   downstream (default) | upstream | both.
    --depth N       Maximum hop distance (default: unlimited).
    --format F      text (indented tree, default) | json | dot (Graphviz).
    --kinds csv     Restrict to base kinds: verifies, derivedFrom, satisfies, supertype,
                    appliesWhen, allocatedTo, refines, derivedFromSafetyGoal, or any
                    declared link-type name (JSON `via` carries the same name) or its
                    `inverse` (selects that type's edges). An unknown name is warned
                    about on stderr, listing the valid names.

Cycles are handled (each element is visited once). Works for qualified names and stable IDs.

## EXIT CODES
    0  traversal printed
    1  usage error: a --direction outside downstream|upstream|both, a --format outside
       text|json|dot, a non-integer --depth, or an unknown option (message on stderr,
       nothing on stdout)
    2  no <qname|id> given

## SEE ALSO
    links, connectivity, n2, follow, link-types
