# check-staged — validate only what is staged, for a pre-commit hook

## SYNOPSIS
    syscribe -m <root> check-staged [--fix-hints] [--json] [--skip-configs]
        [--deny <CODES>] [--max-warnings <N>] [--warnings-as-errors]
    syscribe -m <root> check-staged --install-hook

## DESCRIPTION
A full `validate` can be slow enough that people skip it before committing, and a project gate can
pass while a staged change still introduces errors. `check-staged` runs the same checks as
`validate`, but reports only what the commit touches:

- **Touched elements** are the elements in the staged model files plus the elements that directly
  reference them (their dependants). Staging `.syscribe.toml` or another model-wide config file
  touches every element.
- **Checks** run on the unprojected model and, when the model has a feature model, under each
  `Configuration` a touched element is active in (`--skip-configs` omits the latter for speed).
  Only findings on touched elements are reported.
- **Deletions and renames** can leave a dependant dangling, so for those the check also reports
  every finding the commit introduces compared with `HEAD`, wherever it falls.
- **What is judged** is the staged content: the working tree when it matches the index, otherwise an
  export of the index, so unstaged edits can neither hide nor cause a finding. (An export is made
  in a temporary directory, so `[repos]` peers that are resolved relative to the repository are not
  seen in that case.)
- **No staged model files** — the check decides from `git` alone and exits `0` without loading the
  model, so a commit that stages only code costs almost nothing.

Findings that name a fix say so — `E106` names `syscribe scaffold-gherkin <TC> --fix`. `--fix-hints`
adds a *How to fix* section with each code's catalogue explanation and the commands it points to.

**`--install-hook`** writes `.git/hooks/pre-commit` (wherever `core.hooksPath` puts it) running this
command from the repository root. It refuses to overwrite a pre-commit hook it did not write, and
tells you the line to add to yours instead. If `syscribe` is not on `PATH` when a commit is made the
hook says so and lets the commit through.

## OPTIONS
    --fix-hints           After the findings, explain each code and name the commands that fix it.
    --json                Machine-readable result (schemaVersion 1.0).
    --skip-configs        Check the unprojected model only.
    --deny <CODES>        Comma-separated warning/info codes that fail the check (exit 2).
    --max-warnings <N>    Fail (exit 2) when the warning count exceeds N.
    --warnings-as-errors  Treat every warning as a failure (exit 2).
    --install-hook        Install the pre-commit hook and exit.

## EXIT CODES
    0  nothing staged to check, or no findings that fail the check
    1  an error finding on a touched element, or a usage problem (not a git repository, hook conflict)
    2  a warning gated by --deny / --max-warnings / --warnings-as-errors

## EXAMPLES
    # run from the repository root, after `git add`
    syscribe -m model/ check-staged
    syscribe -m model/ check-staged --fix-hints --deny W002
    # once per clone, to check every commit
    syscribe -m model/ check-staged --install-hook

## SEE ALSO
    validate, scaffold-gherkin, explain, changes
