---
type: Requirement
id: REQ-TRS-SYSMLV2-053
name: "The parser dependency is evaluated for upgrade, its version is reported by syscribe sysml and sysml_submodels, and a drift guard keeps the reported version equal to the pin"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - parser
---

The `sysml-v2-parser` pin shall be evaluated against the newest published release; the outcome (adopted, or deferred with the concrete blockers) shall be recorded in `ADR-SYS-SYSMLV2-001`'s addendum. Whatever version is pinned shall be reported by `syscribe sysml` (a `Parser:` line, and `parser.version`/`parser.astVersion` in `--json`) and by the MCP `sysml_submodels` tool, and a test shall fail when the reported version differs from the version pinned in `crates/syscribe-model/Cargo.toml`, so the report can never silently go stale after an upgrade.

## Rationale

A parser bump changes what ingestion can see (and what `W543` therefore counts as unmapped), so the version is part of the context of every submodel report. The 0.55 release replaced owned `String` names and references with source-span handles resolved through the parsed document, which is a breaking rework of the ingestion layer; recording the evaluation keeps that decision reviewable.
