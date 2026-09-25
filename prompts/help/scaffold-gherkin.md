# scaffold-gherkin — generate/align Gherkin scenarios for a TestCase

## SYNOPSIS
    syscribe -m <root> scaffold-gherkin <TC-id> [--fix]

## DESCRIPTION
Generates or aligns the Gherkin Scenario blocks of a TestCase from its
testFunctions: mappings (so each named function has a matching scenario). With
--fix, writes the alignment back to the file.

## OPTIONS
    --fix   Apply the changes to the TestCase file.

## EXAMPLES
    # needs a TestCase with testFunctions[].scenario entries (none in the bundled models)
    syscribe -m <root> scaffold-gherkin TC-SCHED-BITMAP-001
    syscribe -m <root> scaffold-gherkin TC-SCHED-BITMAP-001 --fix

## SEE ALSO
    who-verifies, validate
