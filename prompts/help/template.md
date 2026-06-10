# template — print a frontmatter skeleton for a type

## SYNOPSIS
    syscribe -m <root> template <type>

## DESCRIPTION
Prints a ready-to-fill YAML frontmatter skeleton for the given element type, with
the required and common fields. Combine with `next-id` and `check-ref` before
writing a new element.

## EXAMPLES
    syscribe -m model/ template Requirement
    syscribe -m model/ template TestCase
    syscribe -m model/ template TestPlan
    syscribe -m model/ template ConfirmationMeasure

## SEE ALSO
    next-id, check-ref, spec types, spec fields
