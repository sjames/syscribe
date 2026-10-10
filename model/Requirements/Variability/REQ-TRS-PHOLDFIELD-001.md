---
type: Requirement
id: REQ-TRS-PHOLDFIELD-001
name: "A whole-value placeholder is allowed in typed numeric and enumerated frontmatter fields"
status: draft
reqDomain: software
reqClass: system
tags:
  - variability
---

A frontmatter field of a fixed list of numeric and enumerated fields shall be able to take its value from a feature parameter, `asilLevel: "{{Features::Safety.level}}"` (GH #268, follow-up to REQ-TRS-PHOLD-001).

## Behavior

- **Fields (v1).** Numeric: `failureRate`, `diagnosticCoverage`, `latentDiagnosticCoverage`, `silLevel`. Enumerated: `asilLevel`, `calLevel`. Any other field keeps treating such a string as an ordinary (usually invalid) value.
- **Syntax.** The YAML value is a string that is exactly one placeholder (`"{{Feature.param}}"`, whitespace around it ignored; it must be quoted in YAML). A `|unit` suffix, or placeholder text mixed with anything else, is not a whole-value placeholder.
- **Base model.** The parser takes a whole-value placeholder out of the field and remembers it (`placeholder_fields`), so the base model validates with the field unset — no type error, no enum error — and every rule that reads the field sees it as unset. Without a matching Configuration (`validate`, `matrix`, `trace`, …) the field stays unset.
- **Projection.** For a selection that matches a stored `Configuration`, the value is resolved as for text placeholders (binding, else fixed `value:`, else `default:`), checked against the field and the parameter's own `range:` / `enumValues:` (which also bind a fixed `value:` or `default:`) and, if valid, stored in the typed field: `failureRate` a finite number ≥ 0; `diagnosticCoverage` and `latentDiagnosticCoverage` a number in 0..=1; `silLevel` an integer 1..=4 (as `E009`); `asilLevel` one of `QM`, `A`, `B`, `C`, `D` (case-insensitive, stored upper-case); `calLevel` one of `CAL1`..`CAL4`. An invalid or unresolved value leaves the field unset.
- **Findings.** The placeholder rules of REQ-TRS-PHOLD-001 apply unchanged to a field placeholder (`E240` gate, `E241` unknown reference, `E242` element active where the feature is not selected, `E243`/`W245` unbound, `W246` runtime parameter). New `E247`: in a configuration where the element is active and the parameter resolves, the value is not valid for the field (the message names the field, configuration, value and what the field accepts).
- **Known limits.** The base model sees the field unset. `E841`–`E843` and `W801` tolerate a placeholder in `asilLevel` / `silLevel`; other rules that read these fields (`W032` on `calLevel`, `W985`/`W033` on coverage and rate, statistics) treat it as unset. Rows exploded from table sheets (TARA, FMEA) and the LSP rename path do not extract field placeholders; use the file-level form.
- A field placeholder counts as a consumer of its parameter (feature card, `refs`/impact) like a text one.
