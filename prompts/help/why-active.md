# why-active — is this element in this product, and why

## SYNOPSIS
    syscribe -m <root> why-active <qname|id> --config <CONF|features> [--json]

## DESCRIPTION
Explains an element's activation under a configuration: its effective appliesWhen
(including conditions inherited from an enclosing package), the relevant feature
selections, and a verdict of active / inactive / always active. --json emits
{element, config, appliesWhen, appliesWhenSource, references, verdict}.

## EXAMPLES
    syscribe -m model/ why-active UAV::Payload::Camera --config CONF-UAV-SURVEY-001

## SEE ALSO
    feature, list --feature, diff, validate --config
