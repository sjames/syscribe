---
type: Package
name: SingleFileFeatureModelExample
---

Worked example (REQ-TRS-FM-005): a full-fledged feature model authored as one
`type: FeatureModel` sheet, exercising every FeatureDef/Configuration
capability — multi-level dotted nesting, mandatory/alternative/or groups with
cardinality, inline and `crossTreeConstraints:` requires/excludes (dotted,
absolute-qname, and stable-id forms, including a reference to a feature
defined the *old* per-file way), typed parameters (range/enum/isRequired/
isFixed/bindingTime/buildVar), `buildExports:`, `parentFeature:` override, a
sheet-level `parameterConstraints:`, and `appliesWhen:`-gated architecture,
requirements and tests.
