# validate — report findings and gate CI

## SYNOPSIS
    syscribe -m <root> validate [--file <path>] [--json]
        [--deny <CODES>] [--max-warnings <N>] [--warnings-as-errors] [--summary]
        [--profile <name>] [--config <C>] [--all-configs]
        [--results <file> [--format <fmt>]] [--fetch-remote]

## DESCRIPTION
Runs the full per-element validation pass and prints the findings (errors and
warnings) only. By default a model with warnings but no errors exits 0; the
gating flags promote chosen warnings to build failures.

## OPTIONS
    --file <path>           Report (and gate on) only the findings for files matching <path>
                            (applies with --config and --all-configs too).
    --json                  Emit findings as a JSON array.
    --deny <CODES>          Comma-separated warning codes treated as gate failures.
    --max-warnings <N>      Fail when the warning count exceeds N.
    --warnings-as-errors    Treat every warning as a gate failure.
    --summary               Print finding counts per code and severity instead of each
                            finding (with --json: [{code, severity, count}]). Gating and
                            the exit code are unchanged. With --all-configs the per-variant
                            result table is already a summary; --summary has no effect there.
    --profile <name>        Apply a named [profiles.<name>] policy from .syscribe.toml
                            (SIL/ASIL-scopable code promotion). See `help` for profiles.
    --config <C>            Project onto a Configuration (id/qname or 'Features::A,Features::B')
                            and validate that variant (escaping refs: E226/W019). The gating
                            flags, --profile and --file apply to the variant's findings.
    --all-configs           Validate every stored Configuration (CI gate). The gating flags,
                            --profile and --file are evaluated per variant (--max-warnings is a
                            per-variant budget); the summary marks each variant pass / gate / error.
    --results <file>        Ingest test results for this run (enables W010), no sidecar write.
                            --format cargo-json|junit|session-log picks the parser (default
                            inferred: .xml → junit, else cargo-json).
    --fetch-remote          Run the .syscribe.toml [remote] hook to fetch remote sourceFiles.

## VARIANT ESCAPES — WHICH CHECK SEES WHAT
A reference from an active element to one gated off in a variant "escapes" it. The three
checks differ in reach:

    validate                 sees no escapes (the 150% model: every element is present)
    validate --config C      checks that one variant; --all-configs checks every stored
                             Configuration
    feature-check --deep     proves the absence of escapes (E227 structural, W020
                             traceability) for every valid configuration, stored or not

In a variant, a structural reference to an inactive element is E226; a traceability or
list-valued reference (satisfies, verifies, derivedFrom, links, ReviewRecord.reviews,
Argument.supports/evidence, PlanningItem evidence ref, ConfirmationMeasure.confirms,
TestPlan.demonstrates/testCases) is the warning W019 — the per-kind "does not resolve"
errors (and E719 for a done leaf PlanningItem) are not raised for a target that exists but
is inactive; a waived evidence entry (with a `rationale:`) raises nothing. An Allocation is
inactive in a variant when any endpoint is, whether named in allocatedFrom/allocatedTo or in
a `features:` entry. A mandatory single reference (FaultTree.topEvent) keeps
its error.

## EXAMPLES
    # Findings table for the whole model
    syscribe -m model/ validate

    # CI gate: fail the build on coverage drift in safety areas
    syscribe -m model/ validate --deny W015,W300,W306

    # Apply a named policy from .syscribe.toml ([profiles.magicgrid] in model_mg/)
    syscribe -m model_mg/ validate --profile magicgrid

    # Certify one product variant
    syscribe -m model/ validate --config CONF-UAV-DELIVERY-001

    # Gate every stored variant in CI, failing on any warning
    syscribe -m model/ validate --all-configs --warnings-as-errors

## EXIT CODES
    0  no errors (and no gate tripped)
    1  one or more Error-severity findings (errors always dominate), or a usage
       error: an undefined --profile, an unresolvable --config, a malformed flag
       value, an unknown option, an unreadable --results file or a --format other
       than cargo-json|junit|session-log. A usage error prints a message to stderr
       and nothing to stdout.
    2  a warning tripped a gate (--deny / --max-warnings / --warnings-as-errors / --profile)

    The contract is the same in every mode. With --all-configs the gate is
    evaluated per variant and the worst variant decides, in the order 1 > 2 > 0:
    1 if any variant has errors, else 2 if any variant tripped a gate, else 0.
    Exit 2 always means "no errors, but a gate failed" — never a usage error.

## SEE ALSO
    audit, verification-depth, matrix, spec validation
