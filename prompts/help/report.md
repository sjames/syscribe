# report — full validation report (the default command)

## SYNOPSIS
    syscribe -m <root> [report]

## DESCRIPTION
With no subcommand, syscribe prints the full 10-section Markdown report: element
inventory, requirement coverage matrix, traceability summary, safety/security
rollups, and the findings tables. For findings only, use `validate`; for a
readiness verdict, use `audit`.

`report` is the default: `syscribe -m <root>` and `syscribe -m <root> report`
are the same command.

## EXAMPLES
    syscribe -m model/
    syscribe -m model/ report > report.md

## EXIT CODES
    0  no error-severity findings (warnings never change the exit code here;
       use `validate --deny/--max-warnings/--warnings-as-errors` to gate them)
    1  one or more error-severity findings

## SEE ALSO
    validate, audit
