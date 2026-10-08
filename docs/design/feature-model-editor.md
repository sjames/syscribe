# Feature model editor and viewer

Status: built through all four phases (0.47.0), 2026-10-08. Not done: an MCP tool for feature edits, editing `parameters:`, structural edits of `featureTree:` sheet entries. Requirements `REQ-TRS-FMED-*`, decision `ADR-SYS-FMED-001`.

## 1. Where things stand

The feature model is well served by the engine and by text. `FeatureDef` elements form a tree
with `groupKind` and `mandatory`, `requires:`/`excludes:` cross-tree constraints, typed
`parameters:`, and `Configuration` elements that select features. `feature-check --deep` proves
void, dead, core and false-optional features with SAT and gives minimal corrections;
`configure` propagates a partial selection into forced and free features and returns an unsat
core when it conflicts; `enumerate_variants` lists valid products; projection (`--config`)
shows one product.

The browser shows none of it. A feature is a row in the model tree, with no tree diagram, no
view of constraints, no analysis, no way to configure a product and no editing beyond the
generic element editor. The model's own authors reach for FeatureIDE, pure::variants or Gears
for exactly these things.

## 2. What industry-leading means here

Judged against FeatureIDE, pure::variants, BigLever Gears and Clafer tooling, the bar is:

1. **A feature diagram people can read.** FODA notation: mandatory and optional marks, XOR and OR
   group arcs, cross-tree constraints as distinct edges, parameters on the node, collapse and
   expand, search, filter and a fit-to-screen overview. Laid out automatically and stable.
2. **Analysis you can see.** Void model, dead, core and false-optional features marked on the
   diagram itself, each with the reason it is so, not only a list of names. Live: it updates when
   the model changes.
3. **A configurator that explains.** Pick and unpick features; the rest propagate at once (forced
   on, forced off, still free); a conflicting pick is refused or shown with the exact
   constraints that clash; the number of valid products is visible; the result is saved as a
   `Configuration`.
4. **Editing that cannot silently break the product line.** Add, rename, delete, regroup,
   reparent, mark mandatory, and draw `requires`/`excludes`, each previewed with its effect on
   validity (this edit makes `X` dead; this one makes the model void) before it is written.
   Undo and redo.
5. **Impact.** For a feature: what it gates (requirements, parts, tests by `appliesWhen`), which
   configurations select it, and what removing it would change.
6. **Comparison.** Two configurations side by side; a feature by configuration matrix.
7. **Scale.** Thousands of features stay smooth, with collapse and search doing the real work.
8. **Export.** The diagram as SVG, PNG, PlantUML and Mermaid, from the browser and the CLI.

Syscribe already has the hard parts (the SAT engine, the diagram IR, the ELK layout, guarded
writes). What is missing is the surface.

## 3. Architecture

- **The feature diagram is a Diagram IR kind.** `DiagramKind::FeatureModel`, derived from a
  `FeatureDef` subtree (or a `FeatureModel` sheet), so the existing ELK layout, sprotty client,
  style, SVG, PlantUML and Mermaid writers and `diagram export` all apply with no second renderer.
  Nodes are features, edges are parent-child, `requires` and `excludes`.
- **A dedicated page, `/features`,** hosts that diagram with panels beside it: Analysis,
  Configurator, Inspector and Impact. It is a page and not a diagram tab because the panels and
  the selection state are the point.
- **All reasoning is server-side.** Analysis, propagation, explanation and counting run in
  `syscribe_model::feature_model` and are exposed as read-only JSON under
  `/api/feature-model/`, the same engine the CLI and MCP use. The browser never solves.
- **Edits are semantic.** `POST /api/feature-model/edit` takes an operation (`add`, `remove`,
  `rename`, `setGroup`, `setMandatory`, `move`, `addConstraint`, `removeConstraint`,
  `setParameter`), maps it onto whichever layout the feature lives in (a file per feature, or a
  `featureTree:` sheet), and goes through the guarded-write engine. A preview mode returns the
  validity delta without writing.
- **Live.** The page re-fetches on the model-reload event, so another agent's or the CLI's edit
  appears at once.

## 4. Phases

1. **Viewer and analysis.** The `FeatureModel` diagram kind, `/features`, the analysis API with
   explanations, overlays on the diagram, search and collapse, export.
2. **Configurator.** Propagation, conflicts with explanation, product count, load and save a
   `Configuration`.
3. **Editing.** Semantic edit operations with a validity preview, undo and redo.
4. **Impact, comparison and scale.** Impact panel, configuration diff and matrix, performance
   work for large models.

## 5. Open questions

- **Explaining a dead feature.** The solver returns an unsat core over assumptions. For a dead
  feature, the explanation is the core of the assumption "this feature is selected" against the
  model's constraints, reported as the human labels the encoding already carries. If the cores
  are too coarse, add a minimal-core pass.
- **Counting products.** Exact counts can be expensive on large models; show the count when the
  solver finishes within a budget and "more than N" otherwise.
