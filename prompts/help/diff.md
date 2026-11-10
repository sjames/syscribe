# diff — what differs between two product variants

## SYNOPSIS
    syscribe -m <root> diff --config <A> --config <B> [--all] [--json]

## DESCRIPTION
Lists the elements active in one configuration but not the other — the delta
between two products of the line. Each --config is a stored Configuration
(id/qname) or an ad-hoc feature set ('Features::A,Features::B'). --json emits
{onlyInA:[…], onlyInB:[…]} (element ids / qualified names), plus `metadata:[{element,onlyInA,onlyInB}]`
when an element in both products applies different stereotypes (a `metadata:` application with its own `appliesWhen:`).

The **parameters** section lists each parameter whose effective value differs between the two
(`~ <FeatureDef>.<param>: A = <value> <unit> (<source>); B = …`), with where each value came from — `own`,
`inherited from <CONF>`, `default`, `fixed`, `derived`, `bindTo` — and `(not set)` for a variant that has
none. Two variants that select the same features but bind different values differ only here. `--all` lists
every parameter, equal ones marked `=`. In `--json` the same rows are `parameters:[{key, unit, differs,
a:{value, source, stage}, b:{…}}]` (always present; `[]` when none differ), and the MCP `diff_configs` tool
returns them too.

## EXAMPLES
    syscribe -m model/ diff --config CONF-UAV-SURVEY-001 --config CONF-UAV-DELIVERY-001
    syscribe -m model/ diff --config CONF-UAV-SURVEY-001 --config CONF-UAV-DELIVERY-001 --all   # every parameter, not only those that differ

## SEE ALSO
    why-active, matrix --features, validate --config
