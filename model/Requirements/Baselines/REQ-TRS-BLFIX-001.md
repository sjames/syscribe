---
type: Requirement
id: REQ-TRS-BLFIX-001
name: "Baseline detail diff works for any layout; verify skips superseded baselines and can list drift"
status: draft
reqDomain: software
reqClass: system
tags:
  - baseline
---

Baseline diff, verify and manifests shall behave consistently for sealed baselines (GH #261, #262).

## Behavior

- A manifest shall record each element's `file` relative to the git root (not an absolute path), so it is portable between machines. A manifest with absolute paths written by an earlier version shall still be usable.
- `baseline diff A B --detail` shall reconstruct each changed element's old and new content with `git show <commit>:<repo-relative path>` for any model-root layout, including a model root that is a subdirectory of the git root.
- `baseline verify` shall report a `superseded` baseline as `skipped (superseded)`, never FAIL, and shall not affect the exit code, consistent with `validate`.
- `baseline diff <BL> --current` and `baseline verify <BL> --detail` shall list added, removed and changed elements between the sealed baseline and the working tree, using the same per-element hash as the seal.
