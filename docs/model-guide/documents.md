# Documents from the model

A design description, a status report or a release note is mostly hand-written text, with
values, tables, requirement text and diagrams from the model at marked places. Syscribe builds
such a document without a generator program per project. There are four pieces, each usable on
its own:

| Piece | What it gives | Command |
|---|---|---|
| Resolved export | Everything a generator would re-derive: bodies, effective parameter values, reverse links, questions, commit | `export --resolved` |
| Value references | A design value in LaTeX, Typst, AsciiDoc or HTML text, by reference | `render <file>` |
| Table views | A table declared in the model: columns, rows, order | `view render --format latex` |
| Templates and `Document` | A whole document with loops, tables and diagrams, and a check that it is up to date | `doc build` |

The worked example is `examples/doc-render/`.

## A document that only needs values

Write the reference in the document and render it. See
[Value references](value-references.md#in-external-documents).

```latex
The heater shall not exceed <<Limits.maxTemp>>.
```

```bash
syscribe -m model/ render doc/chapter.tex --out build/chapter.tex
```

## A document with tables and element text

Declare the document in the model:

```yaml
---
type: Document
name: DesignDescription
status: draft
template: repo:doc/main.tex.tmpl
artefacts: [repo:doc/build/design-description.pdf]
version: "0.2"
audience: customer
---

The design description delivered to the customer.
```

| Field | |
|---|---|
| `status` | Optional. While it is `draft` a template that does not exist yet is not reported (`W505`) |
| `template` | Required. The template; a path resolved like `sourceFile` (`repo:`, `model:`, bare = model root) |
| `rendered` | Where the rendered text goes. Default: beside the template, its name without `.tmpl`, `.j2` or `.jinja` |
| `artefacts` | What a further tool makes of the rendered text, typically a PDF |
| `version`, `audience` | Given to the template as `document.version` and `document.audience` |
| `config` | The Configuration the document is for, when it is for one variant |

Write the template. For a LaTeX result the delimiters are `<< >>` for an expression, `<% %>` for
a statement and `<# #>` for a comment, because braces and `%` belong to LaTeX. Any other result
uses Jinja's own `{{ }}`, `{% %}`, `{# #}`.

```latex
\title{<< document.name >> \\ \large Version << document.version >>}
\date{Commit << meta.commit[:8] >>}

\section{Limits}
The heater shall not exceed << value("Limits.maxTemp") >>.

\section{Parameters}
<< view("Views::Parameters") >>

\section{Requirements}
<% for r in elements(type="Requirement", where="status != draft") %>
\subsection{<< r.id >>: << r.name >>}
<< r | body >>
<% endfor %>

\includegraphics{<< diagram("Diagrams::Context") >>}
```

Build it:

```bash
syscribe -m model/ doc build DesignDescription     # writes doc/main.tex and doc/main.tex.manifest.json
latexmk -pdf -outdir=doc/build doc/main.tex        # the PDF is LaTeX's job
```

### What a template can ask for

| | |
|---|---|
| `value("Elem.attr")` | The value and unit, set for the target (`120\,\textdegree{}C`); `bare=true` for the number alone. An element attribute or a feature parameter |
| `element("REQ-X-001")` | One element: `id`, `name`, `type`, `qname`, `frontmatter.<field>`, `body.raw`, `body.rendered`, `computed.verifiedBy`, `computed.satisfiedBy`, `computed.citedBy`, `questions`, `effectiveParameters` |
| `elements(type=…, where=…)` | The matching elements in qualified-name order; `where` is a condition as in a table view |
| `view("Views::X")` | The table view as a table of the target; `format="latex\|markdown\|csv\|html"`, or `format="data"` for `{columns, rows}` |
| `diagram("X")` | Exports the diagram beside the rendered file and gives its file name; `format="svg\|mermaid\|plantuml\|dot"` |
| `selected("FEAT-X")` | Whether the Configuration selects the feature |
| `meta` | `commit`, `dirty`, `syscribeVersion`, `generatedAt`, `config` |
| `parameters` | The Configuration's effective parameter rows |
| `document` | `name`, `qname`, `version`, `audience`, `config`, `status` |
| `x \| body` | An element's Markdown body in the target's language |
| `x \| quantity` | A `{value, unit}` pair set for the target |
| `x \| latex`, `typst`, `html`, `md`, `safe` | Escape for that language, or print as written |

Everything printed is escaped for the result's language unless a helper or filter produced it.
`include` reads other files beside the template.

### Strict by default

An element, attribute, view or variable the model does not have stops the render, with the
template file and line, and nothing is written:

```
doc/main.tex.tmpl:14: element `Views::Parameters` does not resolve — check the name with `check-ref`
```

So renaming an element the template names fails the build at the line that names it.

### Is the document up to date?

```bash
syscribe -m model/ doc build --all --check
```

```
Documents::DesignDescription: out of date
  - value("Limits.maxTemp") is now 130\,\textdegree{}C
  - view("Views::Parameters") changed
  - the artefact doc/build/design-description.pdf is older than doc/main.tex
```

`doc build` records what the template asked the model for; `--check` asks again and compares. It
also reports a changed template, a rendered file edited by hand, a document never built and a
missing artefact. It writes nothing and exits `2` when a document is out of date, so it can gate
a release.

### One template, several variants

With `config:` on the Document, or `--config` on the command, the template sees that
Configuration: only its active elements, its parameter values, and `selected("FEAT-X")` for a
conditional section.

## Keep the logic out of the template

A template that filters, sorts and applies project conventions becomes the generator again, in a
weaker language. The rule: the template says where a table goes; a
[View](../cli/index.md#views-view-render) says what is in it.

## Validation

| Code | |
|---|---|
| `E508` | A malformed Document: no `template:`, a field of the wrong shape, or a `config:` that is not a Configuration |
| `W505` | A non-draft Document whose template file does not exist |
| `E507`, `W504` | A malformed table view; a column naming an unknown field |
| `W098` | `lint-docs`: a value reference in an external document that does not resolve |
