# ls — list namespace children

## SYNOPSIS
    syscribe -m <root> ls [qname] [--where custom.<key>[<op><value>]]...

## DESCRIPTION
Lists the direct children of a namespace (default: the model root). For the
recursive form, use `tree`.

## OPTIONS
    --where custom.<key>[<op><value>]   Filter to children whose `custom_fields:` match.
                                        `<op>` is `=` (exact; any list element), `=~`
                                        (regex, substring fallback) or `~=` (list
                                        membership); a bare `custom.<key>` tests presence.
                                        Any other operator (`!=`, `==`, `~`, `>`, `<`,
                                        `>=`, `<=`) is a usage error (exit 1).
                                        Repeatable, ANDed.

## EXAMPLES
    syscribe -m model/ ls
    syscribe -m model/ ls UAV::Avionics
    syscribe -m model_mg/ ls SolutionDomain::PhysicalComponents --where custom.mg_layer=physical

## SEE ALSO
    tree, show
