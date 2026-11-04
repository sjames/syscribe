# new — scaffold a feature, a Configuration or a parameter that is right the first time

## SYNOPSIS
    syscribe -m <root> new feature <Name> --under <package|group> [--group optional|alternative|or]
                                          [--mandatory] [--id FEAT-X] [--dry-run] [--json]
    syscribe -m <root> new configuration <CONF-ID> [--from <CONF>] [--stage <stage>] [--feature-model <pkg>]
                                          [--under <pkg>] [--name <text>] [--select <feature>=<bool>]... [--dry-run] [--json]
    syscribe -m <root> new parameter <feature> <name> [--type Real|Integer|Boolean|String] [--unit <u>]
                                          [--range lo..hi] [--enum a,b] [--default <v>] [--required]
                                          [--stage <s>] [--latest <s>] [--dry-run] [--json]

## DESCRIPTION
Creates the file (or extends the `FeatureDef`) so the usual author mistakes cannot happen:

- **feature** — a `FeatureDef` in the directory of a package or of a group feature (`<name>/_index.md`), with a free
  `FEAT-*` id derived from the name (no `E201`; a taken id is extended with the names above), `mandatory:` and the
  `groupKind:` of its own children. A name that is not a basic name is refused.
- **configuration** — a `Configuration` with a valid `CONF-*` id, `status: draft`, the `featureModel:` of its base,
  `derivedFrom:` the base and a `bindingStage:` that is a real stage and not earlier than the base's (`E751`,
  `E753`). `--select` seeds `features:`; without a base the feature model is inferred when the project has one.
- **parameter** — appended to the feature's `parameters:` with a typed, ranged, optionally enumerated entry and a
  binding window whose stages exist and are in order; a default must lie inside the range or enumeration.

The change is previewed on a candidate copy by the guarded-write engine. The report lists the files, the diff and
the findings the change would add or resolve; one that adds an error is refused and nothing is written (exit 1).
Warnings the new element legitimately raises are reported — a feature nobody selects yet is `W024`. `--dry-run`
previews only. The JSON report has the published `scaffold` schema, and the MCP tool `scaffold` takes the same
parameters.

## OPTIONS
    --dry-run   preview only (the global flag)          --json   machine-readable report
Exit 0 on success, 1 when refused or the target is unknown, 2 on a usage error.

## EXAMPLES
    syscribe -m examples/staged-configuration/model/ new feature Gimbal --under Features --dry-run
    syscribe -m examples/staged-configuration/model/ new configuration CONF-UAV-TEST-001 --from CONF-UAV-DEPLOY-001 --stage runtime --dry-run
    syscribe -m examples/staged-configuration/model/ new parameter Features::Telemetry burstMs --unit ms --range 10..500 --default 100 --stage load --latest runtime --dry-run

## SEE ALSO
    bind, template, set, validate, feature-check
