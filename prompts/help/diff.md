# diff — what differs between two product variants

## SYNOPSIS
    syscribe -m <root> diff --config <A> --config <B> [--json]

## DESCRIPTION
Lists the elements active in one configuration but not the other — the delta
between two products of the line. Each --config is a stored Configuration
(id/qname) or an ad-hoc feature set ('Features::A,Features::B'). --json emits
{onlyInA:[…], onlyInB:[…]} (element ids / qualified names).

## EXAMPLES
    syscribe -m model/ diff --config CONF-UAV-SURVEY-001 --config CONF-UAV-DELIVERY-001

## SEE ALSO
    why-active, matrix --features, validate --config
