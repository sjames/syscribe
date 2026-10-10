# mechanisms — what each safety mechanism covers

## SYNOPSIS
    syscribe -m <root> mechanisms [--json] [--uncovered]

## DESCRIPTION
Lists every `SafetyMechanism` (`SM-*`) with the failure modes, requirements and goals it
`covers:`, its `diagnosticCoverage` and `latentDiagnosticCoverage`, `reactionTime`, `safeState`
and `allocatedTo`. `validate` checks the element (`E896`, `E897`) and warns (`W894`) when a
non-draft mechanism's reaction time exceeds the FTTI of a goal it covers, directly or through a
covered requirement's `derivedFromSafetyGoal`.

`--uncovered` lists the `FMEAEntry` rows that no non-retired mechanism lists directly in `covers:` (a draft mechanism counts; an FMEA row reached only through a covered fault-tree event or requirement is still listed).

## OPTIONS
    --json         Machine-readable output.
    --uncovered    List FMEA rows covered by no safety mechanism.

## SEE ALSO
    fmea, metrics, validate
