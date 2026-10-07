---
type: Requirement
id: REQ-TRS-TREX-002
name: "The export can be projected onto a configuration, omitting inactive elements and recording the configuration"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-TREX-000]
breakdownAdr: Decisions::TraceExportADR
tags:
  - traceability
  - export
---

With `--config <C>` (a stored `Configuration` id or qualified name, or an ad-hoc `Features::A,Features::B` set) the export shall project the model exactly as `trace --config` and `export --config` do: requirements, satisfiers, verifiers, parents and children that are inactive in `C` are omitted from every list, and `coverage` is computed over the projected lists. The document's `config` field shall then carry `{ "id", "qname", "name", "activeFeatures": [qualified names] }`. An unresolvable or invalid configuration is a usage error: message on stderr naming it, nothing on stdout, exit 1. Without `--config`, `config` is `null` and nothing is filtered.

## Rationale

Coverage per product-line variant is the question the feature model exists to answer; reusing the projection engine guarantees the same answer `matrix` and `validate --config` give.

## Scope

- Only one configuration per document; a per-variant comparison is the caller's loop.
