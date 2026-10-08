---
type: ADR
id: ADR-SYS-FMED-001
name: "Feature model viewer and editor: the feature diagram is a Diagram IR kind, reasoning stays server-side, edits are semantic and previewed"
status: accepted
tags:
  - feature-model
  - variability
  - visualisation
---

## Context

The feature model has a complete engine (SAT analysis, propagation, explanations, enumeration,
projection) and no browser surface. `docs/design/feature-model-editor.md` sets the bar for an
industry-leading viewer and editor and the architecture below.

## Decision

1. **`DiagramKind::FeatureModel` in the Diagram IR**, derived from a `FeatureDef` subtree or a
   `FeatureModel` sheet. Every existing consumer (sprotty, ELK layout, SVG, PlantUML, Mermaid,
   `diagram export`) renders it with no second renderer. Nodes carry the notation state
   (mandatory, group kind, abstract, parameters, analysis state); edges are parent-child,
   `requires` and `excludes`.
2. **A page, `/features`,** hosts the diagram and its panels (Analysis, Configurator, Inspector,
   Impact), fed by read-only JSON under `/api/feature-model/` computed by
   `syscribe_model::feature_model`.
3. **All solving is server-side**, in the engine the CLI and MCP already use, so the three
   surfaces cannot disagree.
4. **Edits are semantic operations** applied through the guarded-write engine, mapped onto
   either per-file or `featureTree:` layout, with a preview that returns the validity delta
   (newly dead, newly void, newly invalid configurations) before anything is written.
5. **Live**: the page refreshes on the model-reload event.
6. Delivered in four phases: viewer and analysis, configurator, editing, impact and comparison.

## Rationale

Reusing the Diagram IR gives layout, styling and every export for free and keeps one visual
language. Server-side reasoning keeps the browser thin and the engine authoritative. Previewing
the validity effect of an edit is what separates an editor that is safe on a product line from a
text editor with boxes.

## Consequences

- `vis::ir` gains a diagram kind and node/edge vocabulary; a derive generator
  `vis::derive::feature`; spec §8.16 gains the kind.
- `syscribe_model::feature_model` gains explanation and counting entry points that return data,
  not findings text.
- `syscribe-server` gains `/features`, `/api/feature-model/*` and a client module.
