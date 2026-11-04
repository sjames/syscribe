---
type: Requirement
id: REQ-TRS-IMPACTFILES-001
name: "impact starts from changed files or a revision instead of one element"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - git
---

`syscribe impact` shall answer "given these changed files (or this revision range), which requirements, tests and units are affected?" (GH #434).

## Behavior

- `--files <path>...` and `--since <rev>` (the files that differ between `<rev>` and the working tree, including staged, unstaged and untracked files) name the changed files; paths are repository-relative or relative to the working directory.
- Each path maps to the elements bound to it: an element's own file, `implementedBy:` (a file, or a directory covering every file below it), a `TestCase`'s `sourceFile:` and a `PlanningItem`'s `evidence[].path`, resolved with the same rules as `W023`/`E717`. A deleted file still maps.
- The existing traversal runs from the whole set and merges the result; the default reach is both directions to depth 2 (unit → the requirement it satisfies → the tests that verify it), overridable with `--direction`/`--depth`.
- Output groups affected elements by type and marks each `changed` (its own file is in the change set) or not. Files that map to no element are listed separately under *unmapped*, never silently ignored.
- `--fail-on-unreviewed` exits `2` when an affected `Requirement` or `TestCase` was not itself changed.
- `--format text|json` (`dot` is for the single-element form). An unknown revision, a missing path list, an element together with `--files`, or a model outside a git repository exits `1`.
- The MCP `impact` tool accepts `files` and `since` with the same behaviour and the same report.
