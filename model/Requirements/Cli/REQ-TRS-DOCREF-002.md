---
type: Requirement
id: REQ-TRS-DOCREF-002
name: "lint-docs checks external documents of other text types"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - documents
---

`lint-docs` shall check text documents other than Markdown and SVG (GH #530).

1. {#types} In a directory scan it reads `.tex`, `.typ`, `.adoc`, `.html` and `.txt`, or the extensions `[documents] extensions` lists instead. A file named on the command line is read when it is one of those or another known text type.
2. {#checks} Such a file gets the checks that read prose: an unresolvable stable id (`W099`), item and question references (`W092`, `W094`) and, with `[references] copied_values = true`, copied values (`W095`), whose suggestion is written with the file's delimiters.
3. {#w098} A value reference that does not resolve, read with the delimiters of the file type (`REQ-TRS-DOCREF-001#delims`), is `W098`, which fails the run. A feature parameter that only lacks a value outside a Configuration is not reported.
4. {#md} Markdown and SVG files are checked as before.
