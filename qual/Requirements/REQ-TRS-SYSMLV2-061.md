---
id: REQ-TRS-SYSMLV2-061
type: Requirement
name: "Export emits the named-step form for control entries whose name is not the synthesized one"
status: verified
reqDomain: software
verificationMethod: test
---

A \`subActions:\` \`IfAction\`/\`LoopAction\`/\`AssignmentAction\`/\`TerminateAction\` entry whose name differs from the name ingestion would synthesize for its position shall export as \`action <name> { <stmt> }\`, and as the bare statement when the name equals the synthesized one, so both read back as an identical entry (REQ-TRS-SYSMLV2-060). An entry is still written as a comment when it carries fields or a \`loopKind\` ingestion does not read. \`syscribe export-sysml\` over the repository model shall report how many entries still degrade and why.

**Source:** `REQ-TRS-SYSMLV2-061` (product model), `ADR-SYS-SYSMLV2-001`.
