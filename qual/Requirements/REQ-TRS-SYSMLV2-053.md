---
id: REQ-TRS-SYSMLV2-053
type: Requirement
name: "The parser dependency is evaluated for upgrade, its version is reported by syscribe sysml and sysml_submodels, and a drift guard keeps the reported version equal to the pin"
status: verified
reqDomain: software
verificationMethod: test
---

The `sysml-v2-parser` pin shall be evaluated against the newest published release; the outcome (adopted, or deferred with the concrete blockers) shall be recorded in `ADR-SYS-SYSMLV2-001`'s addendum. Whatever version is pinned shall be reported by `syscribe sysml` (a `Parser:` line, and `parser.version`/`parser.astVersion` in `--json`) and by the MCP `sysml_submodels` tool, and a test shall fail when the reported version differs from the version pinned in `crates/syscribe-model/Cargo.toml`, so the report can never silently go stale after an upgrade.

**Source:** `REQ-TRS-SYSMLV2-053` (product model), `ADR-SYS-SYSMLV2-001`.
