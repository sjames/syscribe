---
type: Requirement
id: REQ-TRS-REQMATRIX-001
name: "The Requirements Explorer offers a requirement-by-element matrix lens"
status: draft
reqDomain: software
reqClass: system
tags:
  - server
---

`syscribe-server`'s Requirements Explorer shall present the drawn graph as a matrix of requirements against the elements of one category (GH #269, phase 6; the V-model lens remains).

## Behavior

- A "Matrix view" toggle shows an HTML table whose rows are the drawn `Requirement` nodes and whose columns are the drawn nodes of the category chosen in a "Columns" selector (`tests`, `architecture`, `features`, `safety`, `security`; default `tests`). A cell lists the kinds of the direct edges between the row and the column, in either direction, and is empty when there is none.
- Rows without any cell, and columns without any cell, are kept so gaps are visible; a row with no cell in the chosen category is marked "none". The matrix is a pure function of the drawn (filtered) graph and the category; its headers are `scope`d and every id is a button selecting the element.
- Matrix, table and graph views are mutually exclusive; the matrix is not exported.
