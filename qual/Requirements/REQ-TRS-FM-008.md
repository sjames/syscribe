---
id: REQ-TRS-FM-008
type: Requirement
name: Tool shall accept a dotted tree path in featureTree requires/excludes and shall name the accepted form and closest match for every unresolved feature reference
status: draft
reqDomain: software
verificationMethod: test
---

Inside a single-file `featureTree:` ([[REQ-TRS-FM-005]]) a feature is named by its dotted path relative to the sheet (`Protection.Afci`). Elsewhere it is named by qualified name (`Features::Protection::Afci`) or `FEAT-` id, and a parameter by `<qualified name>.<parameter>`. Authors who guess the wrong form need the finding to tell them the right one.

The tool **shall**:

- resolve an inline `requires:`/`excludes:` entry on a `featureTree:` entry that is a path relative to the sheet (`Protection.Afci`) to the sheet entry it names, as a `crossTreeConstraints:` value does, and leave any other value (a qualified name, a `FEAT-` id, a feature outside the sheet) to resolve as written, so no reference that resolved before changes meaning;
- report a `Configuration` `features:` key that does not resolve to a `FeatureDef` as `E239`, since it selects nothing;
- in each unresolved-feature finding (`E212` requires/excludes, `E209` appliesWhen, `E239` Configuration features, `E222` parameterBindings, `E241` placeholders), name the form that field accepts and, when a `FeatureDef` (or declared parameter) is close, the reference it most likely meant, written in the accepted form;
- report a feature-shaped placeholder that does not parse as `{{<feature>.<param>}}` (for example `{{Features::Mppt.GlobalScan.scanPeriodS}}`) as `E241` rather than leaving it unexpanded and unreported. Placeholder syntax inside code spans and fences, and `{{ … }}` text that is not feature-shaped, stays exempt.

**Source:** GH #325, found while building the grid-tie inverter model (`model_gti/`).

**Acceptance criteria:**

- `requires: [Protection.Afci]` and `excludes: [...]` with a dotted tree path resolve inside the same `FeatureModel` sheet; the qualified-name and `FEAT-` forms still work.
- An unresolved reference in `requires:`, `appliesWhen:`, `Configuration.features:`, `parameterBindings:` and a placeholder each gives a message naming the accepted form and the closest match.
