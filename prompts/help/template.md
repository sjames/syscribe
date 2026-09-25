# template — print a frontmatter skeleton for a type

## SYNOPSIS
    syscribe -m <root> template <type>

## DESCRIPTION
Prints a ready-to-fill YAML frontmatter skeleton for the given element type, with
the required and common fields. Combine with `next-id` and `check-ref` before
writing a new element.

Every element type has a skeleton (the type name is case-insensitive) except
`FMEAEntry`, whose rows are authored inside `template FMEASheet`. Skeletons use the
current schema — e.g. `StateDef` transitions use `source`/`target`/`accept`/`guard`/
`effect` and mark an `isInitial` state; `Baseline` shows the fields `baseline create`
writes (prefer that command, which computes the seal). An unknown type exits 1 and
lists every known type.

## EXAMPLES
    syscribe -m model/ template Requirement
    syscribe -m model/ template TestCase
    syscribe -m model/ template TestPlan
    syscribe -m model/ template StateDef
    syscribe -m model/ template Baseline
    syscribe -m model/ template ConfirmationMeasure

## SEE ALSO
    next-id, check-ref, spec types, spec fields
