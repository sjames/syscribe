# list — enumerate elements of a type

## SYNOPSIS
    syscribe -m <root> list <Type> [scope]
        [--tag <t>] [--feature <F>] [--config <C>]
        [--status <s>] [--sil <v>] [--has-wcet]
        [--where custom.<key>[<op><value>]]... [--json]

## DESCRIPTION
Lists elements of a given type, optionally scoped to a namespace, with filters
for tags, feature gating, lifecycle status, integrity level, and WCET claims.

## OPTIONS
    [scope]          Namespace prefix to restrict to (e.g. UAV::Avionics).
    --tag <t>        Keep only elements whose tags: include t.
    --feature <F>    Keep only elements gated by FeatureDef F (via appliesWhen).
    --config <C>     Project onto a configuration (only active elements).
    --status <s>     Keep only elements whose status: equals s.
    --sil <v>        Keep only elements whose silLevel stringifies to v OR asilLevel == v.
    --has-wcet       Keep only elements that declare a non-empty wcet:.
    --where custom.<key>[<op><value>]   Filter by custom_fields: (op = / != / ~ / > / < /
                     >= / <=; bare custom.<key> = presence). Repeatable, ANDed.
    --json           Emit a JSON array (qualifiedName,type,name,id,status,silLevel,asilLevel,wcet).

## EXAMPLES
    syscribe -m model/ list Requirement
    syscribe -m model/ list PartDef UAV::Avionics
    syscribe -m model/ list Requirement --status draft --sil 4
    syscribe -m model/ list Requirement --has-wcet --json

## SEE ALSO
    types, find, matrix, verification-depth
