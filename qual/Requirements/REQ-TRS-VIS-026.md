---
id: REQ-TRS-VIS-026
type: Requirement
name: Selecting a shape or edge in the diagram editor shows a rendered view of the depicted element's Markdown in a side panel
status: verified
reqDomain: software
verificationMethod: test
---

Clicking a node, port or edge in an open diagram **shall** show the depicted element in a side
panel over the diagram, from `GET /ui/element-card/{*qname}`: its name, type, qualified name,
stable id and status, and the body of its Markdown file rendered by the same renderer as the
detail panel (headings, lists, tables, code, Mermaid blocks), with a control that opens the full
detail dialog. The panel **shall** follow the selection of exactly one shape or edge that has a
`ref`, be left as it is by a multiple selection and by the connect gesture, and have a close
control. A `ref` naming an inline feature **shall** resolve to its nearest owning element: the
card labels it a feature of that owner, lists its declared properties, and shows the documentation
of the feature's type when it resolves, else the owner's. A `ref` naming nothing **shall** show a
card saying so, with the reference text escaped, never an error page.

**Source:** `REQ-TRS-VIS-026` (product model).

**Acceptance criteria:** (a) an element card shows identity and a rendered heading, table, code
block and Mermaid block, and a path to the full detail dialog; (b) a requirement card shows its id
and status; (c) a port feature names itself and its owner, lists `direction` and `typedBy`, and
shows its type's documentation, saying whose it is; (d) a feature without a resolvable type shows
its owner's body; (e) a nested feature path follows `typedBy` to the declaring element; (f) an
unknown reference and an unknown feature of a known element say they are not model elements, with
the reference escaped; (g) the selection logic maps one selected shape or edge to its `ref` at any
nesting depth, and nothing selected, several, a label and a ref-less shape to no change; (h) the
panel markup is hidden by attribute and its ids are present.
