# timing — where the time goes when a model loads and validates

## SYNOPSIS
    syscribe -m <root> timing [--json]

## DESCRIPTION
Loads the model and validates it once, and reports each phase: **load** (walk, parse and the synthesis passes),
**validate**, and **variants** (every Configuration's projected validation, the cost of `validate --all-configs` and
`health`), with the file, element and finding counts and the peak memory. It is the measurement behind the decision
not to build incremental revalidation yet (`docs/design/incremental-validation.md`): run it on your own model and
compare it with the target given there.

The language server revalidates on save, not on every keystroke, so what matters for an editor is `load + validate`.
Build matters a great deal — a release build is several times faster than a debug one — and the report says which
it is.

## OPTIONS
    --json   machine-readable output (schemaVersion 1.0)

## EXAMPLES
    syscribe -m examples/staged-configuration/model/ timing
    syscribe -m model/ timing --json

## SEE ALSO
    validate, health, lsp
