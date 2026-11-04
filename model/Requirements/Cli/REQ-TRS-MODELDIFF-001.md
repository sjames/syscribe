---
type: Requirement
id: REQ-TRS-MODELDIFF-001
name: "changes reports the element-level difference between two git revisions"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - git
---

`syscribe changes <revA> [<revB>]` shall report what changed in the model between two git revisions, or between a revision and the working tree, without a sealed baseline (GH #433).

## Behavior

- Each revision is exported with `git archive` to a temporary directory and loaded like any model; the working tree, index and refs are never modified, and the export is removed on exit.
- Elements are paired by stable `id:` (else qualified name). Each entry is `added`, `removed`, `moved` (same identity at a different qualified name) or `modified`.
- Equality uses the parsed frontmatter and the normalised body, so reformatting YAML or re-wrapping whitespace yields no entry.
- A `modified` entry names the `status` transition, links added/removed (`satisfies`, `verifies`, `derivedFrom`, `achieves`, `appliesWhen`, user `links:`, …), `evidence:` entries added/removed, other changed frontmatter fields and whether the body changed.
- References that stopped resolving, and ones that started to, are listed separately; a reference that vanished with its removed owner is not "resolved".
- `--types` and `--package` restrict both sides identically; `--json` emits `schemaVersion: "1.0"`.
- An unknown revision, a non-git directory, or a model directory absent at a revision exits `1` with a message; success is `0` whether or not anything changed.
