# ls — list namespace children

## SYNOPSIS
    syscribe -m <root> ls [qname] [--where custom.<key>[<op><value>]]...

## DESCRIPTION
Lists the direct children of a namespace (default: the model root). For the
recursive form, use `tree`.

## OPTIONS
    --where custom.<key>[<op><value>]   Filter to children whose `custom_fields:` match
                                        (op = / != / ~ / > / < / >= / <=; bare
                                        `custom.<key>` = presence). Repeatable, ANDed.

## EXAMPLES
    syscribe -m model/ ls
    syscribe -m model/ ls System::Software
    syscribe -m model/ ls --where custom.supplier=Bosch

## SEE ALSO
    tree, show
