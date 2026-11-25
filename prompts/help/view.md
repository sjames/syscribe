# view — materialise a View: the elements its `expose:` selects, as a tree or a document table

## SYNOPSIS
    syscribe -m <root> view render <View|ViewDef> [--format tree|table|json|mermaid|latex|markdown|csv|html]
                                   [--config <C>] [--out <file>]

## DESCRIPTION
A `View` (or `ViewDef`) declares what it shows in `expose:` — a qualified name, an
import pattern (`Pkg::*` direct members, `Pkg::**` the whole subtree) or a map
`{target, isRecursive, filter}` — and may carry a `filterCondition:`. `view render`
evaluates that against the model and prints the selected elements. A view also
inherits the `expose:`/`filterCondition:` of its `typedBy:` ViewDef.

Filters understand `@MetadataName` (the element carries that metadata application),
a field comparison (`type == PlanningItem`, `tag == external-input`, `status != draft`),
`not`, `and`, `or`; any other clause is reported on stderr and treated as true.
Unresolved expose targets are reported on stderr (and as `W502` by `validate`).

The default format follows the view's `rendering:` — `asTreeDiagram` → tree,
`asTableView`/`asElementTable` → table, `asInterconnectionDiagram` → mermaid;
otherwise tree; a view that declares `columns:` defaults to table. Read-only.

## TABLE VIEWS
A view with `columns:` declares the table a document prints of it, so a generator
needs no code per table (GH #531):

    type: View
    name: OpenQuestions
    expose: [{target: "Planning::**", filter: "type == PlanningItem and tag == external-input"}]
    rows: questions            # one row per question, not per element
    where: "status == open"
    columns:
      - {field: ref, heading: Question}          # PI-X#Q1
      - {field: text, heading: Text, width: 6cm}
      - {field: recommended, heading: Recommendation, empty: "—"}
    sort: [ref]
    caption: "Open questions"

    columns   the columns in order: a field name, or {field, heading, empty, align, width};
              {tagPrefix: phase-, heading: Phase} shows `2` for the tag `phase-2`
    rows      one row per entry of a list inside each element: questions, parameters,
              parameterConstraints, features, amendments, … (default: one row per element)
    where     keep matching rows: `<field> == <value>`, `<field> != <value>`, `<field>`,
              joined by and / or / not; a list field matches when one entry equals the value
    sort      a field or a list of them; `-field` descending; digits compare as numbers
    nest      the field naming a row's parent element (`nest: parent`): children follow,
              indented (not with `rows:`)
    marks     [{when: "tag == placeholder", symbol: "†", note: "Placeholder"}]: flag the
              matching rows and print the note under the table
    caption   the caption; `label:` the label a document refers to it by (default
              `tab:<view-name>`, e.g. `tab:open-questions`)

Fields. An element row: any frontmatter field (dotted for a nested map), `custom.<key>`,
and `ref` (id, else qualified name), `id`, `qname`, `type`, `tag`, `summary` (first line of
the body), `body`, `verifiedBy`, `satisfiedBy`, `derivedChildren`, `openQuestions`. A list
row: the keys of its entry, then the element's fields (`element.<field>` names them
outright). A `questions` row adds `ref`; a `parameters` row adds `key`, `effectiveValue`
and `quantity` (the value under `--config`, else fixed, derived or default; `quantity`
with its unit).

Formats: `latex` is a `longtable` with `booktabs` rules, `\caption` and `\label` (it
breaks over pages; load both packages); `markdown` a pipe table; `csv` RFC 4180; `html` a
`<table>`. Each escapes cell text for its target and sets a quantity the target's way
(`120\,\textdegree{}C`). `table` prints the declared columns for a terminal; `json` the
table as data (`columns`, `rows[].cells`). A document format on a view with no
`columns:` prints element, type and name.

`validate` reports a malformed declaration as `E507` and a column, sort key, `nest:` or
condition naming a field no row can have as `W504`.

## OPTIONS
    --format <f>     tree | table | json | mermaid | latex | markdown | csv | html
                     (default from `rendering:`; table for a view with `columns:`)
    --config <C>     evaluate the view in Configuration C (its active elements and its
                     parameter values)
    --out <file>     write a table format to a file instead of stdout

## EXAMPLES
    syscribe -m model/ view render Views::SafetyView
    syscribe -m model/ view render Views::SafetyView --format table
    syscribe -m model/ view render Views::SafetyView --format csv
    syscribe -m examples/doc-render/model/ view render Views::Parameters --format latex
    syscribe -m examples/doc-render/model/ view render Views::Parameters --format markdown --config CONF-HTR-001

## SEE ALSO
    render, export, show, diagram, validate
