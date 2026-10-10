---
type: Requirement
id: REQ-TRS-COMPLY-001
name: "compliance reports which expected work products of a standard exist and are approved"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - audit
---

`syscribe compliance --standard <name>` shall list, per process area of a standard, the expected work product and whether the model has it, approved (GH #239, v1).

## Behavior

- `compliance --standard aspice|iso26262|iso21434 [--config <C>] [--json] [--fail-on-missing]`. The built-in mapping is a small table of process area → expected work product → selector; a `[standards.<name>]` table in `.syscribe.toml` with `[[standards.<name>.item]]` entries (`process`, `workProduct`, `type`, optional `reqClass`, `tag`, `testLevel`) replaces the built-in items of that standard and may define additional standards.
- A selector matches native elements by element `type` plus the optional `reqClass`, `tag` (any) and `testLevel`. With `--config` the model is projected first, so elements gated off are absent.
- Per item: `present` (matches), `approved` (matches whose status is approved, implemented, verified, active, done, completed, accepted or closed) and `status`: `complete` (all matches approved, at least one), `partial` (some present, not all approved), `missing` (none).
- Text output is a table with a summary line; `--json` carries `standard`, `items[{process, workProduct, present, approved, status}]` and `summary{complete, partial, missing}`. An unknown standard or malformed `[standards]` table exits 1. `--fail-on-missing` exits 1 when any item is missing.
