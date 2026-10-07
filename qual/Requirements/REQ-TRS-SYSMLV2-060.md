---
id: REQ-TRS-SYSMLV2-060
type: Requirement
name: "Ingestion preserves a declared name on a single-control-statement named action step"
status: verified
reqDomain: software
verificationMethod: test
---

A nested \`action <name> { <stmt> }\` usage with no typing, subsetting, redefinition or accept/send clause, whose brace body holds exactly one \`if\`/\`while\`/\`loop\`/\`for\`/\`assign\`/\`terminate\` statement, shall ingest as that statement's \`subActions:\` entry carrying \`<name>\` (not a synthesized \`if_1\`-style name). A named step does not advance the synthesized-name counter of its kind, so bare statements keep their positional names. Every other nested action usage is unchanged (a \`PerformAction\`).

**Source:** `REQ-TRS-SYSMLV2-060` (product model), `ADR-SYS-SYSMLV2-001`.
