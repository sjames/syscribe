# capabilities — what this binary supports, in one call

## SYNOPSIS
    syscribe capabilities [--json]

## DESCRIPTION
For a front end that must work against whichever `syscribe` is installed. Needs no model. The JSON
document (`schemaVersion` 1.0) holds the tool `version`, every command with its checked options,
the published JSON contracts with their `schemaVersion` (see `schema`), which optional capabilities are
present (`stagedConfiguration`, `paramsExport`, `variantDiff`, `coverageMatrix`, `environmentMatrix`, …),
the MCP tool names and the custom LSP requests. The same document is returned in the `experimental.syscribe`
field of the LSP `initialize` result.

With `-m <root>` the document also carries `modelMinVersion`: the `[syscribe] minVersion` that model declares
(`null` when none). A model that needs a newer syscribe than this one is refused by every command, so a front
end reads it from the model, not from the version it finds installed.

A capability is `true` when every command it needs exists, so a front end tests the capability it wants rather
than comparing version numbers.

## OPTIONS
    --json    The full document; without it a short summary.

## EXAMPLES
    syscribe capabilities
    syscribe capabilities --json

## SEE ALSO
    schema, version, mcp
