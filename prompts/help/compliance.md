# compliance — which expected work products of a standard the model has

## SYNOPSIS
    syscribe -m <root> compliance --standard <name> [--config <C>] [--json] [--fail-on-missing]

## DESCRIPTION
For each process area of a standard, lists the expected work product and how many matching
model elements are **present** and **approved** (status approved, implemented, verified,
active, done, completed, accepted or closed). Per item: `complete` (at least one, all
approved), `partial` (some present, not all approved) or `missing`.

Built in: `aspice` (ASPICE PAM 3.1: SYS.2/SYS.3/SWE.1/SWE.2/SWE.4/SWE.5/SYS.5/SUP.4/SUP.8; test
levels L1–L3 → SWE.4, L4 → SWE.5, L4/L5 → SYS.5), `iso26262`
(hazardous events, safety goals, technical safety requirements, FMEA, FTA, DFA, confirmation
measures, verification) and `iso21434` (TARA, damage/threat scenarios, cybersecurity goals,
security controls, vulnerability reports). A `[standards.<name>]` table in `.syscribe.toml`
replaces the items of that standard and may define a new one:

    [[standards.aspice.item]]
    process = "SWE.1"
    workProduct = "Software requirements"
    type = "Requirement"          # element type
    reqClass = "system"           # optional
    reqDomain = "software"        # optional
    tag = ["sw"]                  # optional, any of
    testLevel = ["L1", "L2"]      # optional (TestCase); a string or a list

Unknown keys, wrongly typed values, an unknown element `type` and an empty item list are
errors (a typo must not silently widen a selector). Elements that are `retired`,
`deprecated`, `superseded` or `rejected` are not counted.

With `--config` the model is projected first, so elements gated off are absent.

## OPTIONS
    --standard <name>    aspice | iso26262 | iso21434 | a configured name
    --config <C>         Evaluate one variant.
    --json               Items {process, workProduct, type, present, approved, status} and a summary.
    --fail-on-missing    Exit 1 when any item is missing.

Exit 0 · 1 on an unknown standard, a malformed `[standards]` table or `--fail-on-missing`.

## SEE ALSO
    audit, matrix, verification-depth
