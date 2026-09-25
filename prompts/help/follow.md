# follow — traverse one named link, one hop or transitively

## SYNOPSIS
    syscribe -m <root> follow <qname|id> <link> [--reverse] [--transitive]
                                          [--depth <N>] [--format text|json|dot] [--json]

## DESCRIPTION
Walks a single named relationship from an element and reports every element it
reaches, with its hop depth, id/qualified name, type, name, and the element it was
reached from. Read-only.

`<link>` is one of:

- a **user-defined link type** declared in `[linkTypes.<name>]` of `.syscribe.toml`
  (followed forward, source → target — the element holding the `links:` entry is
  the source);
- that type's declared **`inverse`** (followed target → source);
- a **built-in link**: `satisfies`, `verifies`, `derivedFrom`, `refines`,
  `supertype`, `typedBy`, `allocatedTo`;
- a **built-in reverse index**: `satisfiedBy`, `verifiedBy`, `derivedChildren`,
  `refinedBy`, `specializedBy`, `allocatedFrom`.

A built-in name follows only the built-in field itself — never the user-defined
types that `extends` it (use the custom type's own name for those).

Each element is reported once, at its shortest hop distance; the start element is
never reported. Cycles terminate.

## OPTIONS
    --reverse       Flip the direction (a type name followed backwards, an inverse
                    or reverse-index name followed forwards).
    --transitive    Follow to a fixed point (default: one hop).
    --depth N       Bound the traversal to N hops (implies --transitive).
    --format F      text (default) | json | dot (Graphviz digraph).
    --json          Shorthand for --format json.

JSON shape:
    {"start": <id|qname>, "link": <name>, "direction": "forward"|"reverse",
     "results": [{"qname", "id", "type", "name", "depth", "from"}]}

## EXIT STATUS
    0  success (including "no elements reached")
    1  the element does not resolve
    2  usage error, or an unknown link name (the available names are printed)

## EXAMPLES
    # against the bundled link-types example (examples/link-types/model/)
    syscribe -m examples/link-types/model/ follow Architecture::WatchdogMonitor mitigates
    syscribe -m examples/link-types/model/ follow REQ-BRK-003 mitigatedBy
    syscribe -m examples/link-types/model/ follow REQ-BRK-001 derivedChildren --transitive --format json
    syscribe -m examples/link-types/model/ follow REQ-BRK-004 conflictsWith --depth 2 --format dot

## SEE ALSO
    link-types, links, impact, trace, refs
