---
type: Requirement
id: REQ-TRS-BL-004
name: "baseline create seals the scope and writes element and manifest"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BL-000]
breakdownAdr: Decisions::BaselineADR
tags:
  - baseline
  - cli
---

Syscribe shall provide a `baseline create` subcommand that freezes a scope and records it.

- `syscribe -m <root> baseline create --tag <tag> [--name <n>] [--approver <a>]
  [--frozen-scope <sel>] [--id <BL-id>]` shall resolve the scope (REQ-TRS-BL-003), compute
  the seal (REQ-TRS-BL-002), and write a new `Baseline` element (REQ-TRS-BL-001) under
  `model/Baselines/` plus a manifest file `<git-root>/baselines/<id>.manifest.json` (the git
  root discovered from the model path).
- **Scope selector grammar.** `--frozen-scope` (matching the `frozenScope` field name)
  accepts a single string of semicolon-separated `key=value` clauses, where `value` is a
  comma-separated list for the multi-valued keys — e.g.
  `--frozen-scope "package=VehicleSystem::Powertrain;types=Requirement,TestCase;status=approved;tags=safety"`.
  Omitting the flag ⇒ whole-model scope. The parsed clauses populate the `frozenScope` object
  written to the element verbatim, so the CLI form and the authored frontmatter form are
  equivalent.
- **Identity defaults.** When `--id` is omitted, the id shall be derived deterministically
  from the tag (sanitized to the `BL-*` grammar). When `--name` is omitted, the label shall
  default to the tag. If the resolved `id`, `gitTag`, or manifest path **already exists**,
  `create` shall refuse and write nothing.
- **Git anchor.** `create` shall capture the current `HEAD` commit as `gitCommit`, and shall
  expect a **clean working tree**; with uncommitted changes it shall refuse unless
  `--allow-dirty` is given (a dirty tree's commit does not represent the sealed content).
  `create` shall **not** create the git tag itself — tagging is a release action the user
  owns; `create` only records the intended `gitTag` name.
- **Manifest.** The manifest shall be a lean, committed JSON artifact containing: the
  baseline identity and provenance, the resolved scope, `gitCommit`, the tool/schema
  version, the `aggregateHash`, and one entry per in-scope element (stable id, qualified
  name, type, file, status, and per-element hash). It shall additionally embed a **readiness
  snapshot** captured at seal time — the validation error and warning counts and the
  in-scope element counts by type. (Requirement-coverage ratios are deferred to a later
  phase.)
- **Review awareness.** `create` shall warn when an in-scope trace link is **suspect** or has
  **no baseline at all**; `--require-reviewed` shall upgrade both to a refusal that writes
  nothing, so a passing `--require-reviewed` run asserts every in-scope relationship was
  reviewed, not merely that no baselined link went stale.
- The manifest is JSON and therefore ignored by the model walker; the `Baseline` element is
  the version-controlled seal, and the manifest is the assessor's evidence log.
- The manifest records a schema version for forward-compatibility. Reading or migrating an
  **older-version** manifest is **deferred**: v1 tooling assumes the current schema, and a
  version-skew migration path will be specified when the schema first changes.
