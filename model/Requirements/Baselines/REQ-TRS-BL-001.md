---
type: Requirement
id: REQ-TRS-BL-001
name: "Baseline element type and schema"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BL-000]
breakdownAdr: Decisions::BaselineADR
tags:
  - baseline
  - schema
---

Syscribe shall provide a first-class element type **`Baseline`**, authored under
`model/Baselines/` and accumulating as a release history.

## Identity

- `Baseline` is id-identified with the stable prefix **`BL`**, matching the FEAT-style
  relaxed pattern `^BL(-[A-Z0-9]{2,12})+$` — like `FeatureDef`, no forced numeric suffix, so
  a release-style id such as `BL-2026-07` is valid. Additional prefixes (e.g. `REL`) are
  configurable via `[ids.prefixes]`.
- The `id` (model identity) is **distinct** from the version-control tag: the free-form git
  tag string lives in `gitTag:` (e.g. `gitTag: REL-2026-07`) and is never the element id.

## Frontmatter

- `name` — free-prose label (required). When `baseline create` is not given `--name`, it
  shall default the label to the git tag.
- `status` — lifecycle: `draft | approved | released | superseded` (REQ-TRS-BL-005).
- `date` — the baseline date.
- `approver` — the accountable identity that approved the baseline.
- `gitTag` — the intended source-control tag name.
- `gitCommit` — the commit the baseline was sealed at (captured by `create`,
  REQ-TRS-BL-004).
- `frozenScope` — the scope selector (REQ-TRS-BL-003). It is named `frozenScope` (not
  `scope`) because `scope` is already a distinct free-form field on other element types.
- `seal` — a generated block holding `aggregateHash`, `elementCount`, and the `manifest`
  path (REQ-TRS-BL-002, REQ-TRS-BL-004).
- `supersedes` — optional reference to the `Baseline` this one replaces (REQ-TRS-BL-005).

The Markdown body carries the release notes / assessment context. Being a recognized type
with recognized fields, a `Baseline` element shall not raise the unknown-type or
unrecognized-field diagnostics.
