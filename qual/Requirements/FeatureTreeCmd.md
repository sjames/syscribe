---
id: REQ-TRS-DISC-001
type: Requirement
name: Tool shall provide a features command that renders the whole feature model as a tree
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a `features` subcommand that renders the entire feature model as a tree, with indentation following namespace nesting (`Features::Payload::Survey` is shown as a child of `Features::Payload`). This gives an at-a-glance overview of the product line's variability surface without having to open individual `FeatureDef` files.

For each `FeatureDef` node the tool **shall** show:

- its name and qualified name;
- its `groupKind` (e.g. `alternative`, `or`, `and`);
- any `requires:` / `excludes:` cross-tree constraints declared on the feature;
- its typed `parameters:` — for each, the parameter name, its type, and its `range`/`enumValues` if present;
- a "selected in N of M configurations" rollup, where N is the number of `Configuration` elements that select the feature (`true`) and M is the total number of `Configuration` elements.

The command **shall**:

- support a `--json` flag emitting the tree as structured data carrying the same per-node fields;
- be **dormant** when no feature model is present — if the model declares no `FeatureDef`, it **shall** print a clear "no feature model present" notice and exit `0` rather than erroring;
- exit `0` on success.

The command **shall** be discoverable in `--help`.

## Rationale

A product line's feature model is its variability contract, but today it can only be read by opening each `FeatureDef` file and manually correlating it with the `Configuration` elements. A single tree view — with constraints, parameters, and a selection rollup inline — makes the variability surface reviewable in one place and surfaces features that ship in few or no configurations.

**Source:** §9 (PLE); product-line feature discoverability.

**Acceptance criteria:** on a model with a feature model, `features` prints every `FeatureDef` as a tree indented by namespace nesting, each node showing name/qualified name, `groupKind`, `requires:`/`excludes:`, typed `parameters:` (with range/enumValues when present), and a "selected in N of M configurations" rollup; `--json` emits the same data as structured output; on a model with no `FeatureDef` it prints a "no feature model present" notice and exits `0`; the command exits `0` on success and is listed in `--help`.
