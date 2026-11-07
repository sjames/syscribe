---
type: Package
name: CodeTrace
---

Requirements for the reverse of the implementation trace (§12.8.1, GH #451): finding code that no
`implementedBy:` or `sourceFile:` names, under the opt-in `[traceCode]` table, with a shrink-only
baseline, the `W326`–`W328` findings and the per-repository check across `[repos]`.
