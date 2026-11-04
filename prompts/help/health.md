# health — one cheap summary of the project, for a status bar

## SYNOPSIS
    syscribe -m <root> health [--json]

## DESCRIPTION
One validation run feeds every count: errors and warnings (and the five most frequent codes), requirements (total,
orphan `W005`, unverified), stale evidence (suspect links `W090`, stale results `W091`), and the number of
Configurations whose variant fails validation. `status` is `error` when there is an error or a failing Configuration,
`warning` for warnings or unverified requirements, else `ok`. Each non-zero metric lists the command that shows the
detail (`clickThrough`), and `elapsedMs` says what the call cost.

Parameter values files written by `params export` are not registered in the model, so stale ones are not counted
here; `params check <FILE> --config <C>` checks one.

The JSON (schema `health`) is the payload of the MCP tool `health` and the LSP request `syscribe/health`.

## OPTIONS
    --json   machine-readable output

## EXAMPLES
    syscribe -m examples/staged-configuration/model/ health
    syscribe -m examples/staged-configuration/model/ health --json

## SEE ALSO
    validate, requirement-coverage, suspect, capabilities
