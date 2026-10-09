# fmea — FMEA report and analysis commands

## SYNOPSIS
    syscribe -m <root> fmea report [--fmea-sheet <id>] [--json | --format md|html|json]

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
    --format md|html|json   Render the severity x occurrence heat table instead (cell tone
                            from S x O, failure modes marked with their RPN band; html is a
                            standalone page). Not combined with --json.

## EXAMPLES
    # against the bundled automotive model (model_auto/)
    syscribe -m model_auto/ fmea report
    syscribe -m model_auto/ fmea report --fmea-sheet FMEA-ENG-001
    syscribe -m model_auto/ fmea report --json
    syscribe -m model_auto/ fmea report --format html

## SEE ALSO
    fault-tree, hara, validate, spec safety
