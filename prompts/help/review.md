# review — show one review record (ReviewRecord) in detail

## SYNOPSIS
    syscribe -m <root> review <RR-id> [--json]

## DESCRIPTION
Prints the full detail of a single `ReviewRecord` (§19): status, review type, date,
reviewers, the `recordedAt:` pointer to the external review, the covered elements, and
the action items with their dispositions.

## OPTIONS
    --json   Emit JSON instead of Markdown.

## SEE ALSO
    reviews
