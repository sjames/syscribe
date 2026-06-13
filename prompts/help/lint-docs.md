# lint-docs — scan external Markdown for unresolvable stable ID tokens

## SYNOPSIS
    syscribe -m <root> lint-docs <path>... [--json]

## DESCRIPTION
Scans one or more files or directories for tokens that match a stable-ID pattern
(REQ-*, TC-*, ADR-*, FEAT-*, FM-*, FTE-*, AOU-*, SG-*, CM-*) and reports any
that do not resolve to a known element in the loaded model.

Exits non-zero if any unresolvable tokens are found, enabling CI gating.

## OPTIONS
    <path>...    Files or directories to scan (directories are scanned recursively
                 for .md files).
    --json       Emit findings as a JSON array of {file, line, code, token}.

## EXAMPLES
    syscribe -m model/ lint-docs docs/
    syscribe -m model/ lint-docs docs/ --json
    syscribe -m model/ lint-docs docs/adr/ADR-001.md

## NOTES
A W099 warning is emitted for each unresolvable token. Tokens in code fences and
inline code are scanned the same as body text — lint-docs is intentionally
conservative (no false negatives).

## SEE ALSO
    validate, check-ref, refs
