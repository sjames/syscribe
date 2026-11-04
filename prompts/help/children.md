# children — the direct children of a tree node, with counters

## SYNOPSIS
    syscribe -m <root> children [<node>] [--view containment|configurations]
                                [--offset N] [--limit N] [--json]

## DESCRIPTION
One call per expanded node of an explorer tree, so a front end neither walks the directory nor runs a command per
node. Without `<node>` it lists the roots of the view; with one (qualified name or id) its direct children.

`containment` (default) lists packages and elements by qualified name, packages first. Each child has its `id`,
`type`, `status`, `file`, whether it is a `package` and has `children`, and `counters`: `children`, for a
requirement `derivedChildren`, for anything verified `verifyingTests`, and `errors` / `warnings` — those of the
element itself, or for a package those of its whole subtree. A directory with no `_index.md` appears as a
`virtual` package.

`configurations` lists Configurations as a tree: the roots are those that derive from no other and are no one's
sub-configuration; the children of one are the Configurations that name it in `derivedFrom:` (`relation:
derivedFrom`) and those it consolidates in `subConfigurations:` (`relation: subConfiguration`), each with its
`bindingStage`.

Pages hold `--limit` children (default 200, at most 1000); `nextOffset` is the offset of the next page and null on
the last. The JSON (schema `children`) is the payload of the MCP tool `children` and the LSP request
`syscribe/children`.

## OPTIONS
    --view <v>      containment or configurations; anything else exits 2
    --offset <N>    first child to return (default 0)
    --limit <N>     page size
    --json          machine-readable output
An unknown node exits 1.

## EXAMPLES
    syscribe -m examples/staged-configuration/model/ children
    syscribe -m examples/staged-configuration/model/ children Features --json
    syscribe -m examples/staged-configuration/model/ children --view configurations

## SEE ALSO
    ls, tree, show, capabilities
