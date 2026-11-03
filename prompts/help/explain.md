# explain — what a validation code means and how to fix it

## SYNOPSIS
    syscribe explain <code>

## DESCRIPTION
Prints the catalogue explanation of a validation finding (`E106`, `W042`, `I010`, …) and
the commands that explanation points to. Needs no model. It is the command-line form of
the MCP `explain_finding` tool, and `syscribe help <code>` is the same thing.

An unknown code exits 1; `syscribe spec validation` lists every code.

## EXAMPLES
    syscribe explain E106
    syscribe help W042

## SEE ALSO
    validate, spec
