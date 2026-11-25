---
type: Requirement
id: REQ-TRS-DOCREF-003
name: "refs lists the uses of a value in the external documents the project names"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - documents
---

`refs <Elem>.<attr>` shall list the uses of the attribute in the external documents named by `[documents] paths` of `.syscribe.toml` (GH #530), so a change to a value shows which document sections it touches.

1. {#paths} `paths` lists files and directories relative to the model root. A directory is read recursively for Markdown and the `[documents]` text types; each file is read with its type's delimiters.
2. {#section} The uses are printed as *Uses in documents* (file, line, reference as written) and carried in `--json` as `documentUses`.
3. {#param} `refs <Feature>.<param>` lists the uses of a feature parameter in those documents the same way.
4. {#none} With no `[documents]` table, or no use in a document, the output is what it was.
