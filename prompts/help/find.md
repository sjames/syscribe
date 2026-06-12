# find — fuzzy search by name, ID, or content

## SYNOPSIS
    syscribe -m <root> find <pattern> [--where custom.<key>[<op><value>]]...

## DESCRIPTION
Searches element names, stable IDs, and documentation bodies, ranking results by
relevance. Use `extref` to look up by external reference, `list` to enumerate by
type.

## OPTIONS
    --where custom.<key>[<op><value>]   Filter to elements whose `custom_fields:` match.
                                        `<op>` is `=`, `!=`, `~` (contains), `>`, `<`,
                                        `>=`, `<=`; a bare `custom.<key>` tests presence.
                                        Repeatable — multiple `--where` are ANDed.

## EXAMPLES
    syscribe -m model/ find throttle
    syscribe -m model/ find "brake release"
    syscribe -m model/ find . --where custom.supplier=Bosch

## SEE ALSO
    list, extref, show
