# sysml — inspect the SysMLv2 submodels in the model

## SYNOPSIS
    syscribe -m <root> sysml [--json]

## DESCRIPTION
Lists every package declaring `sysmlSubmodel: true` (ADR-SYS-SYSMLV2-001) with the
`.sysml`/`.kerml` files parsed (and whether each parsed), the number of elements
ingested per element kind, the parsed-but-unmapped construct counts (the data behind
the advisory `W543`) and the W540-W543 findings raised for the submodel. Read-only.
With no submodel in the model it says so and exits 0.

## OPTIONS
    --json   {"submodels": [{package, indexFile, fileCount, filesParsed, files:[{path,
              parsed, unmapped}], elementTotal, elementsByKind, unmappedTotal,
              unmapped, findings:[{code, file, message}]}]}

## EXAMPLES
    syscribe -m model/ sysml
    syscribe -m model/ sysml --json

## SEE ALSO
    validate, mcp
