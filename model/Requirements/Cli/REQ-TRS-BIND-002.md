---
type: Requirement
id: REQ-TRS-BIND-002
name: "A violated cross-parameter constraint is an error of validate and is refused by bind"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

A Configuration that violates a `parameterConstraints:` entry (`E221`, or `W025` for `severity: warning`) shall be reported by `validate` — also under `--config` and `--all-configs` — and not only by `feature-check`, so that `bind` and the MCP `bind` tool refuse a value that introduces it exactly as they refuse a value outside `range:` (GH #465; extends `REQ-TRS-BIND-001`).

- The finding is listed once under the added errors of the `bind` report and nothing is written.
- A constraint that is still undetermined because parameters are open is not a violation: it stays allowed, as `residual` already reports it.
- `feature-check` keeps reporting `E221`/`W025` and still adds `E213` and `W014`, which stay `feature-check` only.
