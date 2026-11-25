# render — write a document from the model: value references replaced, or a template rendered

## SYNOPSIS
    syscribe -m <root> render <file> [--out <file>] [--escape <target>]
                              [--open <s> --close <s>] [--config <C>]
    syscribe -m <root> render --template <file> [--out <file>] [--config <C>] [--escape <target>]
    syscribe -m <root> render <diagram-path>

## DOCUMENTS
`render <file>` reads an external text document — LaTeX, Typst, AsciiDoc, HTML,
Markdown, plain text — and writes it with each value reference replaced by the
current value and unit from the model (GH #530). The document states a design
value the way an element body does, `{{Elem.attr}}`, so `refs` and `lint-docs`
see the use and the number is never copied.

A reference names an element (stable id, qualified name or unique basic name)
and an attribute with a single value (`syscribe help refs`), or a feature and
one of its parameters (`{{Limits.maxTemp}}`, `{{FEAT-LIMITS.maxTemp}}`). A
parameter takes its value in the `--config` Configuration; without one, its
fixed, derived or default value. `{{Elem.attr|value}}` writes the bare value
without the unit.

The value is escaped for the target and the unit is set the way the target
writes a quantity:

    target     150 ms          120 degC                   chosen for
    latex      150\,ms         120\,\textdegree{}C        .tex .sty .cls .ltx
    typst      150~ms          120~°C                     .typ
    html       150&#8239;ms    120&#8239;&deg;C           .html .htm .xhtml
    markdown   150 ms          120 degC                   .md
    none       150 ms          120 degC                   anything else

Where `{{ }}` clashes with the document language, choose other delimiters:
`--open '<<' --close '>>'`, or per file type in `.syscribe.toml`:

    [documents]
    paths = ["../doc"]              # what `refs <Elem>.<attr>` scans (files or directories)
    extensions = ["tex", "typ"]     # text types `lint-docs` and `refs` read (default: tex typ adoc html txt)

    [documents.delimiters]
    tex = ["<<", ">>"]

Delimiters are punctuation only. In Markdown a reference inside code is left as
written; in every other language each delimited `X.y` is a reference.

A reference that does not resolve stops the render: each is printed as
`<file>:<line>: E331: …`, nothing is written, exit 1.

## TEMPLATES
`render --template <file>` renders a template with the model as its data: loops over
elements, tables from views, diagrams, conditionals (GH #532). The helpers, the
delimiters and the strict mode are described in `syscribe help doc`; a `Document`
element and `doc build` add a drift check. Use plain `render <file>` when a
document only needs values put in.

## OPTIONS
    --out <file>      Write the result there (directories are created); stdout otherwise.
    --escape <t>      latex | typst | html | markdown | none. Default: by file extension.
    --open <s> --close <s>
                      The delimiters of a reference, both together. Default: the
                      file type's `[documents.delimiters]` entry, else `{{` `}}`.
    --config <C>      Take feature-parameter values, and the model projection, from
                      Configuration C.

## DIAGRAM ELEMENTS
`render <diagram-path>` (a file that is a Diagram element of the model, and none of
the options above) prints the diagram embedded in the element's body to stdout, with
links to the model elements it shows injected. <diagram-path> is the element's file
path (or a path suffix of it), not a qualified name.
- `diagramKind: Mermaid` — the ```mermaid block, with a `click <node> href "<path>" _blank`
  directive added for every `%% link: <NodeId> <qname-or-id>` comment line in the block.
  Nodes are not detected from their labels: a block with no `%% link:` lines gets no
  `click` lines (an unresolved target is warning `W410`).
- any other `diagramKind` — the embedded ```svg block, with `<a href>` wrappers
  added around shapes whose entry in the frontmatter `shapes:` list carries a `link:`.

Mermaid example, inside the block:

    graph TD
      PERF["Mission Performance"]
      %% link: PERF REQ-UAV-PERF-000

It does not lay out or generate a diagram: companion SVG files come from the
browser (Save companion SVG) or from `plantuml render`.

## EXIT CODES
    0  rendered or printed
    1  document: an unresolvable reference, an unreadable file or a usage error
       diagram: no `diagramKind:`, or the expected ```mermaid / ```svg block is missing

## EXAMPLES
    # the worked example: examples/doc-render/ (a LaTeX chapter and its model)
    syscribe -m examples/doc-render/model/ render examples/doc-render/doc/chapter.tex
    syscribe -m examples/doc-render/model/ render examples/doc-render/doc/chapter.tex --config CONF-HTR-001 --out build/chapter.tex
    syscribe -m examples/doc-render/model/ render examples/doc-render/doc/chapter.tex --open '<<' --close '>>' --escape none

    # prints the block unchanged unless it has `%% link:` lines (the bundled one has none)
    syscribe -m model/ render model/Diagrams/RequirementTraceMermaid.md

## SEE ALSO
    doc, view, refs, lint-docs, export, plantuml
