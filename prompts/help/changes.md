# changes — element-level model diff between two git revisions

## SYNOPSIS
    syscribe -m <root> changes <revA> [<revB>] [--types T1,T2] [--package <qname>] [--json]

## DESCRIPTION
Reports what changed in the model between `<revA>` and `<revB>` (default: the
working tree), with no sealed baseline needed. Each revision is exported with
`git archive` into a temporary directory and loaded like any model; the working
tree and index are never touched.

Elements are paired by stable `id:` (else qualified name), so a rename or move
is one `moved` entry. Equality is judged on the parsed frontmatter and the
normalised body — the projection `suspect` and `baseline` hash — so re-indenting
YAML or re-wrapping whitespace produces no entry.

Each entry is `added`, `removed`, `moved` or `modified` and says what changed:
- a `status` transition (`draft → approved`);
- links added/removed (`satisfies`, `verifies`, `derivedFrom`, `achieves`,
  `appliesWhen`, `allocatedTo`, `implementedBy`, `parent`, `blockedBy`, user
  `links:` …);
- `evidence:` entries added/removed;
- other frontmatter fields whose value changed, and whether the body changed.

After the entries, **newly broken** references (a link target that resolved at
`<revA>` and no longer does, or a new element pointing at nothing) and **newly
resolved** ones are listed. The text output is one line per element, suitable
for a pull-request description.

## OPTIONS
    --types T1,T2     Only elements of these types (e.g. Requirement,TestCase).
    --package <qn>    Only elements under this package (qualified name).
    --json            Machine-readable output (schemaVersion 1.0).

## EXIT CODES
    0  success, whether or not anything changed
    1  usage error, unknown revision, not a git repository, or the model
       directory did not exist at a revision

## EXAMPLES
    syscribe -m model/ changes main
    syscribe -m model/ changes v0.56.0 v0.57.0 --types Requirement --json

## SEE ALSO
    baseline diff, suspect, impact, validate --new-since
