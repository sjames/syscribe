---
type: Requirement
id: REQ-TRS-PLUGIN-006
name: "A plugin author can dry-run one plugin and see its raw envelope without merging it into the graph"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLUGIN-000]
breakdownAdr: Decisions::WasmPluginsADR
tags:
  - plugins
  - cli
---

`syscribe plugins run <alias> --dry-run` shall invoke exactly one configured plugin and print its
raw envelope JSON to stdout, without merging the result into the model graph or running
validation.

## Rationale

Plugin execution itself is otherwise fully automatic — every command that loads the model runs
configured plugins as part of the normal `walk_model` pass, with no separate invocation step. That
is the right default for consumers, but a plugin author debugging their own parser needs the
fastest possible loop: see exactly what came back, including when it doesn't parse as valid JSON,
without re-running `validate` and hunting through a `W532` finding's truncated message preview.
