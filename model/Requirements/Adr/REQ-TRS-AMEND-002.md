---
type: Requirement
id: REQ-TRS-AMEND-002
name: "Edits to accepted elements without an amendment are flagged"
status: draft
reqDomain: software
reqClass: system
tags:
  - adr
  - amendments
---

With `[traceability] require_amendments = true` in `.syscribe.toml`, `validate` shall raise `W324` on an `accepted` ADR, or a Requirement at `approved`/`implemented`/`verified`, whose normative content changed since the commit that set that status and that has no amendment dated on or after the change (GH #454).

## Behavior

- The status commit is the oldest commit of the newest unbroken run of commits of the element's file (`git log --follow`, at most `[agents] audit_limit`, default 200) in which the element is at a guarded status.
- Normative content is the suspect projection: body plus frontmatter without editorial fields, `status` and `amendments`.
- The latest change is an uncommitted working-tree edit (dated today, UTC) when the working tree differs from `HEAD`, else the newest commit in the run whose projection differs from its predecessor's (its author date).
- No git, no repository, or a status set only in the working tree: nothing is reported. Off by default; a malformed value is `W632`.
