---
id: REQ-TRS-SYSMLV2-065
type: Requirement
name: "Compound unit expressions ingest with or without spaces and export as unit expressions, not quoted names"
status: verified
reqDomain: software
verificationMethod: test
---

A literal-with-unit whose bracketed unit is a compound expression (\`[N * m]\`, \`[m/s]\`, \`[kg*m/s^2]\`) shall ingest into \`unit:\` as the whitespace-free expression text; ingestion shall never silently drop the unit of a literal whose unit is such an expression. Export shall write a unit that is a compound expression of names as \`[N*m]\` (an expression), and only quote a unit that is not expressible as one; both parse back to the same \`unit:\`.

**Source:** `REQ-TRS-SYSMLV2-065` (product model), `ADR-SYS-SYSMLV2-001`.
