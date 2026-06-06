---
id: REQ-TRS-CONF-002
type: Requirement
name: Configuration Selection Parsing Is Robust and Visible
title: Tool shall parse Configuration feature selections consistently, surface empty selections (W016), and display them
status: draft
reqDomain: software
verificationMethod: test
---

A `Configuration`'s feature selections are declared as a `features:` **map** of `<FeatureDef qualified name>: true/false` (§9.8). To prevent the silent-ignore footgun where a mis-authored selection block yields an all-N/A coverage matrix, the tool **shall**:

1. **Template ⇄ parser agreement** — `template Configuration` **shall** emit the canonical `features:` map (the only selection syntax honoured by `matrix` and `appliesWhen` evaluation), not a form the parser ignores.
2. **No silent ignore (`W016`)** — when a feature model exists (at least one `FeatureDef`) and a `Configuration` parses **zero** feature selections — e.g. because it used an unrecognised `selections:` key — the tool **shall** emit warning `W016` identifying the configuration, rather than silently selecting no features. `W016` is **not** emitted when no feature model is present.
3. **Visibility in `show`** — `show <Configuration>` **shall** display the parsed feature selections (and explicitly state when none parsed), so a parse failure is visible locally rather than only as a downstream all-N/A matrix.

**Source:** Issue #12

**Acceptance criteria:** `template Configuration` output contains a `features:` map and no `selections:` key; a `Configuration` whose only selection block is the legacy `selections:` form, under a model with a `FeatureDef`, produces a `W016` finding while a `features:`-map configuration does not; the same legacy configuration with no `FeatureDef` present produces no `W016`; `show <Configuration>` lists the selected features for a valid configuration and reports "none parsed" for an empty one.
