# search-text — ranked full-text search (BM25)

## SYNOPSIS
    syscribe -m <root> search-text <query> [--json] [--limit <N>]
        [--type <T>] [--status <s>] [--config <C>]

## DESCRIPTION
Ranked full-text search over the normative text of elements (Markdown body +
name), scored with Okapi BM25 and returned best-first with a snippet marking the
first hit. Complements `find`/`search` (which fuzzy-match identifiers): use this
to locate elements by what they say, ordered by relevance. The index is built in
memory from the model — no external service, no persisted index.

## OPTIONS
    --limit <N>    Return at most N results (default 10), ordered by score.
    --type <T>     Restrict to one element type (e.g. Requirement).
    --status <s>   Restrict to elements with status: <s>.
    --config <C>   Search only the elements active in a Configuration (id/qname or
                   'Features::A,Features::B'). Unresolvable → exit 1.
    --json         Emit one { total, results } document (results ordered by score).

## EXAMPLES
    syscribe -m model/ search-text "thermal shutdown"
    syscribe -m model/ search-text "watchdog" --type Requirement --limit 5
    syscribe -m model/ search-text "encryption" --json

## EXIT CODES
    0  ok    1  usage error (empty query, or unresolvable --config)

## SEE ALSO
    find, stats, digest, list
