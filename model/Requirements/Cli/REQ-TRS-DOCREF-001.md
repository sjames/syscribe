---
type: Requirement
id: REQ-TRS-DOCREF-001
name: "render writes an external document with its value references replaced"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - documents
---

`render <file> [--out <file>] [--escape <target>] [--open <s> --close <s>] [--config <C>]` shall write an external text document with each value reference replaced by the current value and unit from the model (GH #530), so a document in LaTeX, Typst, AsciiDoc, HTML or plain text uses the model's own reference syntax (`REQ-TRS-ATTRREF-001`).

1. {#refs} A reference is `<open>Elem.attr<close>`: an element (stable id, qualified name or unique basic name) and an attribute with a single value, or a feature and one of its parameters. A parameter takes its value in the `--config` Configuration, and without one its fixed, derived or default value. `|value` after the path writes the value without its unit.
2. {#escape} The value is escaped for the target and the unit written as the target writes a quantity: `latex` (`150\,ms`, `120\,\textdegree{}C`), `typst`, `html`, `markdown`, `none`. The target follows the file extension; `--escape` overrides it; an unknown target is a usage error naming the valid ones.
3. {#delims} The delimiters are `{{` `}}`, the entry of the file's extension in `[documents.delimiters]` of `.syscribe.toml`, or `--open` and `--close` given together. A delimiter is punctuation only.
4. {#strict} A reference that does not resolve is reported as `<file>:<line>: E331: …` with the reason; the command exits 1 and writes nothing. In a Markdown file a reference inside code is left as written.
5. {#out} The result goes to `--out` (missing directories are created), else to stdout.
6. {#config} A malformed `[documents]` entry is printed as a warning and ignored.
7. {#diagram} `render <diagram-path>` on a Diagram element, with none of these options, behaves as before.
