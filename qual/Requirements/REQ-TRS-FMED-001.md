---
id: REQ-TRS-FMED-001
type: Requirement
name: A feature model is shown as a feature diagram in FODA notation, laid out automatically, with collapse, search, cross-tree constraints and parameters
status: verified
reqDomain: software
verificationMethod: test
---

`syscribe-model` **shall** derive a diagram of kind `FeatureModel` from a `FeatureDef` subtree, a `FeatureModel` sheet or a package of features, and the browser **shall** show it at `/features`: each feature a node with a mandatory or optional mark, abstract state and parameters; children joined by tree edges with an XOR or OR wedge for `groupKind: alternative` or `or`; `requires` and `excludes` as distinct dashed overlay edges that take no part in layout. Layout is automatic, with parents centred over their children and several roots side by side. A subtree **shall** collapse and expand, a feature be found by name or id and revealed, and the model fit to the window. The diagram **shall** export as SVG, PlantUML and Mermaid from the browser and from `diagram export`.

**Source:** `REQ-TRS-FMED-001` (product model).

**Acceptance criteria:** (a) a `FeatureModel` diagram with a package or feature subject derives the right nodes, tree edges and constraint edges, and a subject of another type is `W418`; (b) notation state (mandatory, group kind, child count, id) is carried to the sprotty model and constraints are flagged overlay and carry no labels; (c) Mermaid, PlantUML and SVG render it, with marks and wedges in the SVG; (d) `/features` and the diagram and export endpoints are served; (e) collapse hides a subtree and the edges that touch it and counts what it hid, search matches name, id and qualified name, and revealing a match expands exactly its collapsed ancestors; (f) the toolbar, canvas and script ids the page needs are present.
