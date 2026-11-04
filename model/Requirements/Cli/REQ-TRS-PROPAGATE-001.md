---
type: Requirement
id: REQ-TRS-PROPAGATE-001
name: "propagate reports what a partial feature selection forces, without writing"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

`syscribe propagate [--from <Configuration>] [--select <feature>=<bool>]... [--stage <stage>] [--json]`, the MCP tool `propagate` and the LSP request `syscribe/propagate` shall report, for a partial selection, the features forced on, forced off and free, whether the choices can be completed (else the smallest clashing choices and the constraints they clash with), the number of products that remain and one completion (GH #416).

- Nothing is written. For a Configuration's own selection the forced and free sets equal those of `configure`.
- `--stage` leaves features bound at another stage out of the lists and marks them `deferred`, as `configure --stage` does.
- The three surfaces return one payload, with the published `propagate` schema. The engine is the one the web configurator uses; its responsiveness on a 2,000-feature model is bounded by `REQ-TRS-FMED-007`.
