---
type: Requirement
id: REQ-TRS-DRYRUN-001
name: "Every command that writes files honours a global --dry-run"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - git
---

A global `--dry-run` shall make any command that writes files leave the tree byte-identical (GH #444, `ADR-SYS-PROVENANCE-001`).

## Behavior

- User-facing file writes (model edits, sidecars, generated exports, caches, the pre-commit hook) go through one write layer; under `--dry-run` a write is recorded as "would write" and not performed. The flag may precede or follow the command.
- A command's status line never claims a write that did not happen ("Would write" for "Wrote").
- For each of the writing commands — `set` (including `question.*`), `claim`, `release`, `move`, `applies-when`, `scaffold-gherkin --fix`, `baseline create`, `suspect accept`, `ingest-results`, `import-reqif`, `plantuml`, `summarize`, `export-html`, `diagram export`, `export-reqif`, `trace-export`, `sbom`, `magicgrid --svg`, `params export`, `agent-guide --write` and `check-staged --install-hook` — a run changes the tree, and the same run with `--dry-run` does not.
