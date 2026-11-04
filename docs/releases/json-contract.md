# JSON contract changelog

The `--json` forms a front end depends on are published contracts (GH #413): each carries a `schemaVersion` and has a
JSON Schema, printed by `syscribe schema <name>` and checked against the real output in CI. `syscribe capabilities
--json` (also the `experimental.syscribe` field of the LSP `initialize` result) lists them with their versions.

**Rules.** Adding an optional property is compatible and does not change the version. Removing or retyping a property,
or changing what it means, bumps `schemaVersion` and is listed here. Clients ignore properties they do not know.

| Contract | Command | Version |
|---|---|---|
| `features` | `features --json` | 1.0 |
| `feature` | `feature <name> --json` | 1.0 |
| `configure` | `configure <Configuration> --json` | 1.0 |
| `residual` | `residual --config <Configuration> --json` | 1.0 |
| `matrix` | `matrix --json` | 1.0 |
| `diff` | `diff --config <A> --config <B> --json` | 1.0 |
| `feature-check` | `feature-check [--deep] --json` | 1.0 |
| `params-check` | `params check <FILE> --config <Configuration> --json` | 1.0 |
| `propagate` | `propagate --json`, MCP `propagate`, LSP `syscribe/propagate` | 1.0 |
| `explain-feature` | `explain-feature --json`, MCP `explain_feature`, LSP `syscribe/explainFeature` | 1.0 |
| `model-changed` | LSP notification `syscribe/modelChanged` | 1.0 |

## Additive changes within 1.0

- `matrix` gained `gaps` (GH #424): one entry per cell needing work, with the action to take, the linked tests and, for `create-test`, a suggested test id. Existing properties are unchanged.

## 1.0 — first published version

- `features`, `feature`, `configure`, `residual` and `diff` gained `schemaVersion: "1.0"`; the other fields are unchanged.
- `params check` gained `--json`; its exit codes are unchanged (`1` on errors, `2` when a `--deny`-ed code is raised).
- `syscribe schema` and `syscribe capabilities` were added.
