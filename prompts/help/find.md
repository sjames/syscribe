# find — fuzzy search by name, ID, or content

## SYNOPSIS
    syscribe -m <root> find <pattern> [--where custom.<key>[<op><value>]]...

## DESCRIPTION
Searches element names, stable IDs, and documentation bodies, ranking results by
relevance. Use `extref` to look up by external reference, `list` to enumerate by
type.

## OPTIONS
    --where custom.<key>[<op><value>]   Filter to elements whose `custom_fields:` match.
                                        `<op>` is `=` (exact; any list element), `=~`
                                        (regex, substring fallback) or `~=` (list
                                        membership); a bare `custom.<key>` tests presence.
                                        Any other operator (`!=`, `==`, `~`, `>`, `<`,
                                        `>=`, `<=`) is a usage error (exit 1).
                                        Repeatable — multiple `--where` are ANDed.

## EXAMPLES
    syscribe -m model_auto/ find throttle
    syscribe -m model_auto/ find "position sensor"
    syscribe -m model_mg/ find . --where custom.mg_layer=logical

## SEE ALSO
    list, extref, show
