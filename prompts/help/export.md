# export — structured model graph

## SYNOPSIS
    syscribe -m <root> export [--ndjson] [--config <C>]

## DESCRIPTION
Emits a versioned, machine-readable dump of the whole model — every element with
its frontmatter — as one JSON document (default) or NDJSON (one element per
line). For an element-rooted subgraph with connections, use `connectivity`.

## OPTIONS
    --ndjson      One JSON object per line instead of a single document.
    --config <C>  Project onto a configuration (export only active elements).
    --json        Accepted and ignored — JSON is already the default.

## EXAMPLES
    syscribe -m model/ export > model.json
    syscribe -m model/ export --ndjson
    syscribe -m model/ export --config CONF-UAV-SURVEY-001

## SEE ALSO
    connectivity, validate
