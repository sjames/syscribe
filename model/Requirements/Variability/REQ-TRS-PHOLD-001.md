---
type: Requirement
id: REQ-TRS-PHOLD-001
name: "Feature parameters are usable as double-brace placeholders in element text"
status: draft
reqDomain: software
reqClass: system
tags:
  - variability
---

Element text shall be able to reference a feature parameter so the value is stated once, in the feature model, and every configuration sees its own value (GH #264, #265).

## Behavior

- **Syntax.** `{{Features::Path::Feature.param}}`, with the existing dotted parameter reference; `{{...|unit}}` appends the parameter's `unit:`. The reference may name the feature by qualified name or `FEAT-*` id.
- **Where (v1).** An element's Markdown body and its `name`.
- **Base model.** Kept symbolic and validated as it stands.
- **Projection.** `project()` for a selection that matches a stored `Configuration` replaces each placeholder with the configuration's binding (`parameterBindings:`), else the parameter's fixed `value:`, else its `default:`. A placeholder that cannot be resolved is left as written. Every command that projects (`--config`, `validate --config`, `--all-configs`, reports) therefore sees the substituted text.
- **Opt-in gate.** `E240` when a placeholder is used but the model has no `FeatureDef` or no `Configuration`.
- `E241` the reference names no feature or the feature declares no such parameter.
- `E242` the element is active in a configuration that does not select the owning feature (its `appliesWhen` does not imply the feature).
- Unbound, no fixed value, no default in a configuration where the element is active and the feature selected: `W245` for a draft/review element, `E243` for an approved/implemented/verified one.
- `W246` a `runtime` binding-time parameter is referenced (it has no value at projection time).
