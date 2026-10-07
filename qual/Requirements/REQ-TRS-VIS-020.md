---
id: REQ-TRS-VIS-020
type: Requirement
name: The Requirement generator derives requirements with their derive, refine, satisfy and verify relationships from a Package or requirement subject
status: verified
reqDomain: software
verificationMethod: test
---

For a derived `diagramKind: Requirement` whose subject is a `Package` (or `LibraryPackage`/
`Namespace`), `RequirementDef` or `Requirement`, `vis::derive::requirement` **shall** produce:

- one `Requirement` node per native `Requirement`, SysML `RequirementDef` or `Requirement` whose
  qualified name is the subject or lies under it (any depth), labelled by `name` (or id), with
  compartment lines for the stable `id` and `status` when present;
- a `Derive` edge from each requirement to each `derivedFrom:` target on the diagram and a
  `Refine` edge for each `refines:` target;
- a `Satisfy` edge from a `Block` context node (the satisfying architecture element, drawn with
  its real type's stereotype) to the requirement for every element in the model whose
  `satisfies:` names a requirement on the diagram, and a `Verify` edge from a `TestCase` context
  node for every `TestCase` whose `verifies:` names one;
- a `Containment` edge from a `RequirementDef` to each requirement it owns, when both are on the
  diagram.

`include:`/`exclude:` **shall** apply to requirements and to context nodes alike (by qualified
name, stable id or short name), an entry naming nothing being `W417`. Shape ids **shall** be
`derived_shape_id(<qualified name>)`. A subject of any other type **shall** be `W418` with an
empty graph. Layout hints **shall** be layered, top-to-bottom, with `Derive`, `Satisfy`,
`Verify` and `Refine` reversed so parents sit above. The Mermaid, SVG and PlantUML writers
**shall** accept the generated graph.

**Source:** `REQ-TRS-VIS-020` (product model).

**Acceptance criteria:** through the real walker and validator on a fixture with a parent
requirement, a `RequirementDef` owning two derived children (one also refining the other), a
satisfying `PartDef` and a verifying `TestCase`: (a) a package subject yields one requirement
node per requirement at any depth with `id = …`/`status = …` compartment lines, derive edges
child → parent (targets named by id or qualified name), a refine edge, containment edges from
the `RequirementDef`, a satisfy edge from a `part def` block and a verify edge from a
`test case` node, matching the golden IR `tests/vis_snapshots/derived/reqs.json`; (b) a
`RequirementDef` subject roots the tree at itself and a `Requirement` subject (by stable id)
draws itself and its own satisfier; (c) `include:` by stable id and short name keeps only the
named requirements and context nodes, a context node whose requirement is absent draws nothing,
and a stray entry is one `W417`; `exclude:` drops the named nodes and their edges; (d) a
`PartDef` subject is `W418` with an empty graph; (e) `render_mermaid` yields a `classDiagram`
with `<<requirement>>` classes, compartment members and `«deriveReqt»`/`«verify»` dependencies,
`render_svg` succeeds with dashed keyword-labelled edges and header bands, and `render_plantuml`
yields classes with the id/status body and never a class for a compartment.
