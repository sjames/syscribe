---
type: Requirement
id: REQ-TRS-DOCGEN-001
name: "render --template renders a document template with the model as its data"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - documents
---

`render --template <file> [--out <file>] [--config <C>] [--escape <target>]` shall render a text template with the model as its data (GH #532), for a document that is mostly hand-written text with values, tables, element text and diagrams at marked places.

1. {#engine} The template language is Jinja, by an embedded engine. A LaTeX result uses the delimiters `<< >>`, `<% %>`, `<# #>`; any other result `{{ }}`, `{% %}`, `{# #}`; `[documents.templateSyntax.<ext>]` of `.syscribe.toml` chooses others for a result type. The result type is that of `--out`, else of the template name without its `.tmpl`, `.j2` or `.jinja` suffix. `include` reads files beside the template.
2. {#strict} An unknown element, attribute, view, argument or variable stops the render with `<template>:<line>: <reason>`; the command exits 1 and writes nothing.
3. {#helpers} The template can call `value("Elem.attr", bare=…)` (the quantity set for the result type, resolved as in `REQ-TRS-DOCREF-001`), `element(ref)` and `elements(type=…, where=…)` (elements in the shape of the resolved export, `REQ-TRS-EXPRES-001`), `view(name, format=…)` (a table view, `REQ-TRS-VIEWTAB-002`, as a table of the result type or as data), `diagram(name, format=…)` (the Diagram exported beside the result, giving its file name) and `selected(feature)`.
4. {#vars} The variables `meta` (commit, dirty, syscribe version, time, config) and `parameters` (the Configuration's effective parameter rows) are defined.
5. {#escape} Every printed value is escaped for the result type unless a helper or filter produced it. The filters `latex`, `typst`, `html` and `md` escape for that language; `body` gives an element's Markdown body in the result's language; `quantity` sets a `{value, unit}` pair.
6. {#config} With `--config <C>` the template sees Configuration `C`: its active elements and its parameter values.
