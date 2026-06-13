# fmea — FMEA report and analysis commands

## SYNOPSIS
    syscribe -m <root> fmea report [--fmea-sheet <id>] [--json]

## DESCRIPTION
Sub-commands for FMEA (Failure Mode and Effect Analysis) analysis.

`fmea report` renders an FMEA risk table for all FMEASheet elements in the model,
sorted by RPN (Risk Priority Number) descending so the highest-risk entries appear
first. An optional `--fmea-sheet` flag restricts the output to entries within the
named sheet.

## OPTIONS
    report                  Render FMEA table sorted by RPN descending.
    --fmea-sheet <id>       Restrict to entries in this FMEASheet (id or qname).
    --json                  Emit a JSON array of FMEA entry objects.

## EXAMPLES
    syscribe -m model/ fmea report
    syscribe -m model/ fmea report --fmea-sheet FM-KERN
    syscribe -m model/ fmea report --json

## SEE ALSO
    fault-tree, validate, spec safety
