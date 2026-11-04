# propagate — what a partial selection forces, without writing a file

## SYNOPSIS
    syscribe -m <root> propagate [--from <Configuration>] [--select <feature>=<true|false>]...
                                 [--stage <stage>] [--json]

## DESCRIPTION
Treats a set of feature choices as a *partial* assignment and reports, from the feature model, which other
features are **forced on**, **forced off** or still **free**, whether the choices can be completed at all, the
smallest clashing set of choices and the constraints they clash with when they cannot, how many valid products
remain, and one complete product consistent with the choices. Nothing is written.

`--from` seeds the choices from a Configuration's own `features:` — the result then equals `configure` for it.
`--select` adds or overrides single choices; a feature is named by qualified name or `FEAT-*` id.

`--stage <stage>` is the lens `configure --stage` applies: a feature whose binding window does not contain the stage
is `deferred` (decided at another stage) and left out of `forcedTrue` / `forcedFalse` / `free`; its entry under
`features` keeps its true state and gains `"deferred": true`.

The JSON (schema `propagate`, see `syscribe schema propagate`) is the same payload the MCP tool `propagate` and the
LSP request `syscribe/propagate` return. Unknown features are listed under `unknown`.

## OPTIONS
    --from <Configuration>   seed the choices from this Configuration (id or qualified name); unknown exits 1
    --select <f>=<bool>      one choice; repeatable; later wins; malformed exits 2
    --stage <stage>          see above; unknown stage exits 2
    --json                   machine-readable output

## EXAMPLES
    # against the bundled staged-configuration model
    syscribe -m examples/staged-configuration/model/ propagate --from CONF-UAV-BUILD-001
    syscribe -m examples/staged-configuration/model/ propagate --select FEAT-UAV-TELEM=true --json
    syscribe -m examples/staged-configuration/model/ propagate --from CONF-UAV-BUILD-001 --stage compile

## SEE ALSO
    configure, feature-check, residual, capabilities
