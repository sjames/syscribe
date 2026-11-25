# doc — build the documents the model declares from their templates

## SYNOPSIS
    syscribe -m <root> doc list [--json]
    syscribe -m <root> doc build <Document>... | --all [--check] [--config <C>]

## DESCRIPTION
A `Document` element makes a document part of the model: a design description, a
status report, a release note that is mostly hand-written text, with values, tables,
element text and diagrams from the model at marked places (GH #532).

    type: Document
    name: DesignDescription
    template: repo:doc/main.tex.tmpl          # the template (required)
    rendered: repo:doc/build/main.tex         # default: beside the template, name without .tmpl
    artefacts: [repo:doc/build/design-description.pdf]   # what a further tool makes of it
    version: "0.2"
    audience: customer
    config: CONF-X                            # when the document is for one variant

`doc build <Document>` renders the template and writes the rendered file, and beside
it `<rendered>.manifest.json`: what the template asked the model for. syscribe writes
text; a PDF is made from it by the tool that owns that step (LaTeX, Typst, pandoc).

`doc build --check` writes nothing and reports a document that is out of date:

    - a value, element, element list, view or diagram the template used has changed
      (a value is shown with what it is now), or no longer resolves
    - the template changed, or the Configuration it is built for
    - the rendered file is missing, or was changed after it was built
    - an artefact is missing, or older than the rendered file

`doc list` shows every Document with its template, rendered file and artefacts.

## TEMPLATES
The engine is minijinja (Jinja syntax): `<< expr >>` prints, `<% if %>` / `<% for %>`
control, `<# #>` comments, `include` reads beside the template. For a LaTeX result
the delimiters are `<< >>`, `<% %>`, `<# #>`, because braces and `%` belong to
LaTeX; for any other result Jinja's own (`{{ }}`, `{% %}`, `{# #}`). Choose others
per result type in `.syscribe.toml`:

    [documents.templateSyntax.typ]
    variable = ["<<", ">>"]
    block = ["<%", "%>"]
    comment = ["<#", "#>"]

It is strict: an unknown element, attribute, view or variable stops the render with
`<template>:<line>: <reason>`, and nothing is written. Everything printed is escaped
for the result's language (by its file extension) unless a helper or filter made it.

    value("Limits.maxTemp")              the value and unit, set for the target (120\,\textdegree{}C);
                                         bare=true for the number alone. An element attribute or a
                                         feature parameter, as in `syscribe help render`
    element("REQ-X-001")                 one element: id, name, type, qname, frontmatter.<field>,
                                         body.raw / body.rendered, computed.verifiedBy / satisfiedBy /
                                         citedBy / derivedChildren, questions, effectiveParameters
                                         (`syscribe spec export`)
    elements(type="Requirement", where="status != draft")
                                         the matching elements in qualified-name order; `where` is
                                         a condition as in a table view (`syscribe help view`)
    view("Views::OpenQuestions")         the table view, as a table of the target (a longtable in
                                         LaTeX); format="latex|markdown|csv|html", or format="data"
                                         for {columns, rows} to walk
    diagram("Context")                   exports the Diagram beside the rendered file and gives its
                                         file name; format="svg|mermaid|plantuml|dot" (default svg)
    selected("FEAT-ABS")                 whether the Configuration selects the feature

    meta                                 commit, dirty, syscribeVersion, generatedAt, config
    parameters                           the Configuration's effective parameter rows ([] without one)
    document                             name, qname, version, audience, config, status (doc build)

    x | body                             an element's Markdown body (or a Markdown string) in the
                                         target's language: paragraphs, emphasis, lists, code
    x | quantity                         a {value, unit} pair set for the target
    x | latex   x | typst   x | html   x | md      escape for that language
    x | safe                             print as written

The template says where a table goes; a View says what is in it. Keep filtering,
sorting and project conventions in views, not in templates.

`render --template <file> [--out <file>] [--config <C>]` renders a template with no
Document element (and no manifest); `document` is not defined there.

## OPTIONS
    --all            build or check every Document of the model
    --check          report what is out of date; write nothing
    --config <C>     build for Configuration C (overrides the Document's `config:`):
                     its active elements and its parameter values
    --json           (list) one object {schemaVersion, documents}

## EXIT CODES
    0  built, or every checked document is up to date
    1  a template did not render (nothing written), or a usage error
    2  --check found a document out of date

## EXAMPLES
    syscribe -m examples/doc-render/model/ doc list
    syscribe -m examples/doc-render/model/ doc build HeaterReport --check
    syscribe -m examples/doc-render/model/ doc build --all --check

## SEE ALSO
    render, view, export, spec export, diagram
