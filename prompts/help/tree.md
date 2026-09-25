# tree — recursive namespace tree

## SYNOPSIS
    syscribe -m <root> tree [qname]

## DESCRIPTION
Prints the containment tree under a namespace (default: the model root), one
indented line per element with its type. For the connection/relationship graph
(not just containment), use `connectivity`.

## EXAMPLES
    syscribe -m model/ tree
    syscribe -m model/ tree UAV::Avionics
    syscribe -m model_auto/ tree System::Software

## SEE ALSO
    ls, connectivity
