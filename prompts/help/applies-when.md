# applies-when — set, modify, or clear an element's appliesWhen gate

## SYNOPSIS
    syscribe -m <root> applies-when <element>                      # read: show the gate
    syscribe -m <root> applies-when <element> --set "<expr>" [--dry-run]
    syscribe -m <root> applies-when <element> --clear [--dry-run]

## DESCRIPTION
Adds, replaces, removes, or **displays** the `appliesWhen:` field of a model element
(a Requirement, a Package, a Part/PartDef, or any element that may legally carry it).
The element is resolved by qualified name or stable id.

With **neither --set nor --clear** the command is a read-only display: it prints the
element's **own** appliesWhen and its **effective** condition — the own gate, or, when
the element declares none, the gate inherited from its nearest ancestor package
(transitive package conditioning), naming that package; an element with no gate anywhere
is reported as always applying. `--json` emits `{element, own, effective, inheritedFrom}`.
(This is the static condition; `why-active <element> --config <CONF>` evaluates it for a
specific product.)

On --set the expression is parsed with the appliesWhen boolean grammar
(and / or / not / parentheses; a bare name or list = AND). Every operand must
resolve to a FeatureDef by its qualified name OR its FEAT-* id — the two forms are
interchangeable. The edit is refused (nothing written) if the expression is
malformed or has an unresolved operand (E209), or if the placement is forbidden
(E228: a FeatureDef, a Configuration, the model-root package, a package whose
subtree contains features, or a path that already declares appliesWhen).

After a successful --set, the feature model is checked for bad configurations
(the feature-check --deep analysis: void model, dead features, invalid
configurations). If the model admits a bad configuration the command exits
non-zero, so a gate is never silently applied on top of an unsound feature model.

Only the `appliesWhen:` key is changed; every other byte of the file is preserved.

## OPTIONS
    (no flag)        Read mode: show the element's own and effective gate.
    --set "<expr>"   Set/replace the gate (by FEAT-* id or FeatureDef qualified name).
    --clear          Remove the gate (no-op if absent).
    --json           Read mode: emit the gate as a structured object.
    --dry-run        Validate and preview without writing (still runs the check).

## EXAMPLES
    syscribe -m model/ applies-when REQ-UAV-NAV-001
    syscribe -m model/ applies-when REQ-UAV-NAV-001 --set "FEAT-QUAD"
    syscribe -m model/ applies-when REQ-UAV-NAV-001 --set "Features::Propulsion::Quad"
    syscribe -m model/ applies-when UAV::Payload::Camera --set "FEAT-SURVEY or FEAT-MAPPING"
    syscribe -m model/ applies-when REQ-UAV-NAV-001 --clear --dry-run

## SEE ALSO
    feature-check, configure, features, why-active
