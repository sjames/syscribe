# export — structured model graph

## SYNOPSIS
    syscribe -m <root> export [--resolved] [--ndjson] [--config <C>]

## DESCRIPTION
Emits a versioned, machine-readable dump of the whole model — every element with
its frontmatter — as one JSON document (default) or NDJSON (one element per
line). For an element-rooted subgraph with connections, use `connectivity`.

`--resolved` adds what a document generator needs and would otherwise re-derive
from the files: each element's Markdown `body` (raw, and with value references
rendered), `computed.satisfiedBy` and `computed.citedBy`, a PlanningItem's
`questions` and `openQuestions`, a Configuration's `effectiveParameters` (value,
unit, range, binding time, source, and the expression of a derived one), and a
top-level `meta` (commit, dirty, syscribe version, time, config). With
`--config <C>` the bodies hold that Configuration's values and the top level
carries its `effectiveParameters`. The schema is `syscribe spec export`.

## OPTIONS
    --resolved    Add bodies, effective parameter values, reverse links,
                  questions and the build identity (see above).
    --ndjson      One JSON object per line instead of a single document.
    --config <C>  Project onto a configuration (export only active elements).
    --json        Accepted and ignored — JSON is already the default.

## EXAMPLES
    syscribe -m model/ export > model.json
    syscribe -m model/ export --ndjson
    syscribe -m model/ export --config CONF-UAV-SURVEY-001
    syscribe -m model/ export --resolved --config CONF-UAV-SURVEY-001 > doc-input.json

## SEE ALSO
    connectivity, validate, spec export
