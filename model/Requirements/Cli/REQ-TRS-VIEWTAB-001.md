---
type: Requirement
id: REQ-TRS-VIEWTAB-001
name: "A View declares the table a document prints: columns, rows, filter, order"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - documents
  - view
---

A `View` or `ViewDef` shall be able to declare the table a document prints of the elements it exposes (GH #531), so a document generator needs no code per table.

1. {#columns} `columns:` lists the columns in order. A column is a field name, or a map with `field`, `heading` (default: the field name in words), `empty` (the text for a row with no value), `align` (`left`, `right`, `center`) and `width`; or a map with `tagPrefix` and `heading`, which shows the rest of the element's first tag with that prefix.
2. {#rows} `rows: <list>` gives one row per entry of that list inside each exposed element (`questions`, `parameters`, `parameterConstraints`, `features`, any list of the frontmatter) instead of one row per element.
3. {#fields} An element row answers to its frontmatter fields, `custom.<key>`, and `ref`, `id`, `qname`, `type`, `tag`, `summary`, `body`, `verifiedBy`, `satisfiedBy`, `derivedChildren` and `openQuestions`. A list row answers to the keys of its entry, then to the element's fields; `element.<field>` names the element's. A `questions` row has `ref` (`PI-X#Q1`); a `parameters` row has `key`, `effectiveValue` and `quantity`, the value being that of the `--config` Configuration, else the fixed, derived or default value.
4. {#where} `where:` keeps the rows that match a condition: `<field> == <value>`, `<field> != <value>` or `<field>`, joined by `and`, `or`, `not`; a list field matches when one of its entries equals the value. The same comparisons are understood in `expose:` filters and `filterCondition:`.
5. {#sort} `sort:` orders the rows by one or more fields, `-field` descending, a run of digits comparing as a number. Without it rows are in qualified-name order, entries in file order.
6. {#nest} `nest: <field>` places a row after the row of the element the field names, one level deeper.
7. {#marks} `marks:` (`{when, symbol, note}`) puts `symbol` after the first cell of each row matching `when`, and the note once under the table.
8. {#caption} `caption:` and `label:` give the table's caption and the label a document refers to it by (default `tab:<view name in kebab case>`).
9. {#default} A view with `columns:` prints as a table by default. A view without them is rendered as before.
