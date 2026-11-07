---
type: Requirement
id: REQ-TRS-ATTRREF-001
name: "Prose and scenarios reference a named attribute value with {{Elem.attr}}"
status: draft
reqDomain: software
reqClass: system
tags:
  - references
  - attributes
---

An element body shall be able to state a design value by reference, `{{Elem.attr}}`, instead of
copying the number (GH #452).

1. {#syntax} **Syntax.** `Elem` is resolved by stable id, then exact qualified name, then a basic name
   (last qualified-name segment) that exactly one element has; `attr` names an attribute of it,
   as a dotted path where values nest. An optional `|unit` suffix is accepted; the unit is always
   rendered.
2. {#forms} **Attribute forms.** A single scalar value held as: an inline `features:` entry's `value:`
   (else `default:`), nested `features:` entries by dotted path, an `Attribute` child element's
   `value:` (an `AttributeDef` child's `default:`), a stereotype tagged value `Stereotype.key`
   (defaults applied; `key` alone when only one application has it), or a budget/timing field of the
   element (`ram`, `flash`, `nvm`, `stack`, the `*Capacity` fields, `margin`, `period`, `deadline`,
   `priority`, `cpuMargin`, `wcet`, `failureRate`, `diagnosticCoverage`).
3. {#where} **Where.** References resolve in the Markdown body outside code: inline code spans and
   fenced blocks keep the text literal, except fenced `gherkin` / `feature` blocks, which hold
   TestCase scenarios and are resolved (inline code spans inside them stay literal).
4. {#features} **Feature parameters.** A reference whose element is a `FeatureDef`, or a `::` path or
   `FEAT-` id that resolves to nothing, stays a feature-parameter placeholder (`E240`/`E241`).
5. {#render} **Rendering.** `show` and `export-html` render the value as written followed by its
   `unit:` when it has one (`47244 B`); the file keeps the raw form. The MCP element read adds the
   rendered body next to the raw one when they differ.
