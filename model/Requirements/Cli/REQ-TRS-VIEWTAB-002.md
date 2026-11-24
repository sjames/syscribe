---
type: Requirement
id: REQ-TRS-VIEWTAB-002
name: "view render writes a table view as LaTeX, Markdown, CSV or HTML"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - documents
  - view
---

`view render <View> --format latex|markdown|csv|html [--config <C>] [--out <file>]` shall write the table a view declares (`REQ-TRS-VIEWTAB-001`) in a form a document includes (GH #531).

1. {#latex} `latex` is a `longtable` with `booktabs` rules, the header repeated on each page, `\caption` and `\label`, a `p{<width>}` column for a column with `width:`, nested rows indented and mark notes in the table foot.
2. {#markdown} `markdown` is a pipe table with the caption above it and column alignment.
3. {#csv} `csv` is RFC 4180: a header row, and a field quoted when it holds a comma, a quote or a line break.
4. {#html} `html` is a `<table>` with `<caption>`, `<thead>` and `<tbody>`.
5. {#escape} Each format escapes cell text for its target, and writes a quantity (value and unit) the way the target does.
6. {#others} `table` prints the declared columns for a terminal and `json` the table as data (`columns`, `rows[].cells`, `notes`). A document format on a view with no `columns:` prints the element, type and name.
7. {#config} `--config <C>` evaluates the view in Configuration `C`. `--out <file>` writes the result to a file. An unknown format is a usage error naming the formats.
