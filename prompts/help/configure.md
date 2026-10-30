# configure — assisted configuration

## SYNOPSIS
    syscribe -m <root> configure <Configuration> [--json]

## DESCRIPTION
From a partial selection in a Configuration, reports satisfiability and which
features are forced vs free. Exits non-zero if the partial selection is
contradictory. A Configuration with a `derivedFrom:` base is completed from its
effective selection (the base's selections overlaid by its own, spec §9.8).

Also lists every effective parameter value with its provenance — own, inherited
from a named `derivedFrom:` ancestor, a `subConfigurations:` tier, `bindTo:`, a
derived expression, fixed, or default — and the stage it was bound at
(`bindingStage:`, ADR-SYS-BT-001). `--json` carries it as a `provenance` object
(`bindingStage`, `selections`, `parameters`).

## EXAMPLES
    syscribe -m model/ configure CONF-UAV-DELIVERY-001

## SEE ALSO
    feature-check --deep, features, validate --config
