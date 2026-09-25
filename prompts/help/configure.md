# configure — assisted configuration

## SYNOPSIS
    syscribe -m <root> configure <Configuration> [--json]

## DESCRIPTION
From a partial selection in a Configuration, reports satisfiability and which
features are forced vs free. Exits non-zero if the partial selection is
contradictory. A Configuration with a `derivedFrom:` base is completed from its
effective selection (the base's selections overlaid by its own, spec §9.8).

## EXAMPLES
    syscribe -m model/ configure CONF-UAV-DELIVERY-001

## SEE ALSO
    feature-check --deep, features, validate --config
