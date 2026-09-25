# who-verifies — which test cases cover a requirement

## SYNOPSIS
    syscribe -m <root> who-verifies <req-id> [--config <C>]

## DESCRIPTION
Lists the TestCases that verify a requirement, with their test level, Gherkin
scenario count, and status.

## OPTIONS
    --config <C>    Configuration lens (REQ-TRS-PROJ-001): answer over only the
                    elements active in configuration C (a stored Configuration
                    id/qname or an ad-hoc 'Features::A,Features::B' set).
                    Inactive elements are omitted; if the start element itself
                    is inactive in C the command exits 1 naming it.

## EXAMPLES
    syscribe -m model/ who-verifies REQ-UAV-SAFE-001

## SEE ALSO
    trace, verification-depth, why
