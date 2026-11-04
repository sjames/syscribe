---
type: Requirement
id: REQ-TRS-HEALTH-001
name: "health summarises the project in one cheap call for a status bar"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

`syscribe health [--json]`, the MCP tool `health` and the LSP request `syscribe/health` shall return one summary of the project (GH #428): error and warning counts with the most frequent codes; requirements (total, orphan, unverified); stale evidence (suspect links, stale results); the number of Configurations whose variant fails validation; an overall `status` (`error` for an error or a failing Configuration, `warning` for warnings or unverified requirements, else `ok`); and, for each non-zero metric, the command that lists the detail.

- One validation run feeds every count and the call reports what it cost (`elapsedMs`). Over MCP and LSP the answer is kept per model version and marked `cached`; a reload or change recomputes it.
- Budget: a few seconds in a debug build on the largest example (about 1.4 s measured); a status bar polls the cached answer. Parameter values files are not registered in the model, so stale ones are not counted (`params check` checks one).
- The payload has the published `health` schema. Nothing is written.
