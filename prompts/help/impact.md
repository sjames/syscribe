# impact — change impact analysis over the traceability graph

## SYNOPSIS
    syscribe -m <root> impact <qname|id> [--direction downstream|upstream|both]
                               [--depth <N>] [--format text|json|dot] [--kinds <csv>]
    syscribe -m <root> impact (--files <path>... | --since <rev>) [--direction D] [--depth N]
                               [--format text|json] [--fail-on-unreviewed]

## DESCRIPTION
Traverses the traceability graph (§17) from a named element and reports every reachable
node, its hop distance, and the edge kind that connects it — "if I change this, what else
may need to change?". Read-only.

**Downstream** follows reverse links (who depends on me): `specializedBy`, `derivedChildren`,
`verifiedBy`, `satisfiedBy`, `refinedBy`, `conditionalOn`, `allocatedFrom`,
`safetyGoalChildren`. **Upstream** follows forward links (what I depend on): `supertype`,
`derivedFrom`, `verifies`, `satisfies`, `refines`, `models`, `allocatedTo`, `derivedFromSafetyGoal`.

User-defined links (`links:`, declared in `[linkTypes]` of `.syscribe.toml`) are traversed
too: upstream along the link (source → target, labelled with the type name), downstream
against it (labelled with the type's `inverse`, or `<type> (inbound)`).

**From changed files.** During review the question is usually the reverse: *given these changed
files (or this revision range), what is affected?* `--files <path>...` and `--since <rev>` (the
files that differ between `<rev>` and the working tree — staged, unstaged and untracked) map each
path to the elements bound to it, run the traversal from that whole set and merge the result:

- an element's own `.md` file;
- `implementedBy:` (a file, or a directory — every file below it), resolved like `W023`
  (`repo:`/`model:` prefixes, otherwise model-root-relative);
- a `TestCase`'s `sourceFile:`;
- a `PlanningItem`'s `evidence[].path`.

The default reach is both directions to depth 2 — unit → the requirement it satisfies → the tests
that verify it; widen it with `--depth`. The report lists the mapping, then the affected elements
grouped by type, each marked `changed` (its own file is in the same change set) or `not changed`,
so a reviewer sees what was updated and what was not. Files that map to no element are listed
separately under *Not mapped*, never silently dropped. `--fail-on-unreviewed` exits `2` when an
affected requirement or test was not itself changed — a CI gate that wants the model to move with
the code. `--format dot` and `--kinds` are for the single-element form.

## OPTIONS
    --files <path>...  Changed files (repository-relative or relative to the working directory).
    --since <rev>      Files that differ between <rev> and the working tree.
    --fail-on-unreviewed  Exit 2 if an affected Requirement/TestCase was not itself changed.
    --direction D   downstream (default) | upstream | both.
    --depth N       Maximum hop distance (default: unlimited; 2 with --files/--since).
    --format F      text (indented tree, default) | json | dot (Graphviz).
    --kinds csv     Restrict to base kinds: supertype, derivedFrom, verifies, satisfies,
                    refines, models, allocatedTo, derivedFromSafetyGoal, appliesWhen, or any
                    declared link-type name (JSON `via` carries the same name) or its
                    `inverse` (selects that type's edges). An unknown name is a
                    usage error (exit 1) whose message lists the valid names.

Cycles are handled (each element is visited once). Works for qualified names and stable IDs.

## EXIT CODES
    0  traversal printed
    1  usage error: a --direction outside downstream|upstream|both, a --format outside
       text|json|dot, a non-integer --depth, or an unknown option (message on stderr,
       nothing on stdout)
    2  no <qname|id> given; or, with --fail-on-unreviewed, an affected requirement/test was not changed

## EXAMPLES
    syscribe -m examples/planning-item/model/ impact REQ-RTH-001 --direction both
    syscribe -m examples/planning-item/model/ impact --files examples/planning-item/model/Requirements/REQ-RTH-001.md

## SEE ALSO
    links, connectivity, n2, follow, link-types, changes, check-staged
