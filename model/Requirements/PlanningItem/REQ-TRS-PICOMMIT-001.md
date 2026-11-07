---
type: Requirement
id: REQ-TRS-PICOMMIT-001
name: "A PlanningItem names the commits that delivered it as evidence, checked against git"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
---

A `PlanningItem` `evidence:` entry shall be able to name a git commit, which `validate` checks against the history of the repository containing the model, so a typo or a rebased-away hash is caught and "which commits delivered this item?" can be answered (GH #455).

## Behavior

- An entry `commit: <hash>` (7–40 hexadecimal characters, an abbreviation or a full object name) sits beside the existing `ref:`/`path:` forms and takes the same per-entry waiving `rationale:`.
- `E728`: a non-waived `commit:` is malformed (not hexadecimal, fewer than 7 or more than 40 characters), or — resolved against the git repository containing the model root — is unknown, is an ambiguous abbreviation, or names an object that is not a commit. The message says which. A malformed hash is reported whether or not git is available; resolution is skipped silently when git cannot be run or the model is in no repository.
- Git is asked once per run for all distinct hashes (`git cat-file --batch-check`, then one `git log` for subjects); a resolved commit is cached for the life of the process.
- A resolving commit counts as resolving evidence for `E719`. Where resolution is skipped, a well-formed commit counts too (it cannot be held against the item).
- `[planning] check_commit_paths = true` raises `W320` when a resolving commit entry touches none of the paths its item names — its `evidence: - path:` entries and, for each `evidence: - ref:`, the element's own file, `implementedBy:` and `sourceFile:` (a directory matches any file below it). An item naming no local path is not judged.
- `[planning] require_commit_evidence = true` raises `W321` on a leaf `PlanningItem` at `status: done` with no non-waived commit evidence that resolves (or is well formed where resolution is skipped).
- `steps --json` lists each node's commit entries (`commits: [{commit, sha?, subject?, status}]`) and `context` shows the resolved full sha and subject of each commit entry.
- `set <PI> evidence.add commit=<hash>` appends the entry after checking it to the `E728` bar, refusing a malformed, unknown or ambiguous hash with nothing written.
- `log --item <PI>` (and the MCP `log` tool's `item`) lists the commits a PlanningItem names as evidence.
