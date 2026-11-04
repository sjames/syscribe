---
type: Requirement
id: REQ-TRS-VALBASE-001
name: "validate gates only on findings new relative to a baseline"
status: draft
reqDomain: software
reqClass: system
tags:
  - validation
  - ci
---

`syscribe validate` shall be able to report and gate only the findings that are absent from a baseline, so a model carrying accepted findings can be gated on "no new findings" (GH #431).

## Behavior

- `--new-since <git-rev>` validates the model at `<rev>` and at the working tree and treats a finding as pre-existing when the revision had it. `--baseline-file <file>` does the same against a list saved by `--write-baseline <file>`. The two sources are mutually exclusive, and neither combines with `--all-configs`.
- A finding is identified by code, owning element (stable `id:` else qualified name) and a hash of its message — not by file or line. Moving or renaming a file does not make its findings new. Matching is a multiset: a second copy of an accepted finding is new.
- Only new findings are gated and, in text mode, listed; the count of pre-existing ones is stated. Exit codes keep their meaning (`1` errors, `2` tripped gate, `0` clean), judged over the new findings.
- With a baseline, `--json` lists every finding with `"new": true|false`; without one the output is unchanged.
- `--write-baseline` saves every current finding and does not change the run's own gating.
- With `--config <C>` the same variant is judged at the earlier revision.
- An unknown revision, an unreadable or malformed baseline file, or a conflicting option exits `1`.
