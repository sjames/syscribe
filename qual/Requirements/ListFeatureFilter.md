---
id: REQ-TRS-DISC-004
type: Requirement
name: list --feature Filter
title: Tool shall extend list with a --feature filter restricting to elements gated by a feature
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** extend the `list <type>` command with a `--feature <Features::X>` filter that restricts the listing to elements whose `appliesWhen:` expression names feature `X` as an operand — the "what does this feature gate" filter. An element matches when `X` appears anywhere in its `appliesWhen:` expression (bare name, AND-list, or boolean expression per [[REQ-TRS-VAR-003]]).

The `--feature` filter **shall**:

- be **orthogonal** to the existing `--tag` filter — when both are supplied, an element must satisfy both to be listed (AND semantics);
- be **orthogonal** to the `--config` projection lens — `--feature` selects by which feature gates an element, independent of any projected variant;
- accept the feature as a qualified name (`Features::X`);
- exit non-zero with a clear message if `X` does not resolve to a `FeatureDef`.

The flag **shall** be discoverable in the `list` command's `--help`.

## Rationale

[[REQ-TRS-DISC-002]] shows what a single feature gates from the feature's side. The same question often arises from the listing side — "show me only the `Part`s gated by `Features::DualFlightController`". A first-class filter on `list` makes the reverse `appliesWhen:` index queryable per element type, and being orthogonal to `--tag` and `--config` lets it compose with the existing filters rather than duplicating them.

**Source:** §9 (PLE); product-line feature discoverability; extends `list <type>` and reuses the `appliesWhen:` operand index from [[REQ-TRS-DISC-002]].

**Acceptance criteria:** `list <type> --feature Features::X` lists only elements of that type whose `appliesWhen:` names `Features::X` as an operand; combining `--feature` with `--tag` lists only elements satisfying both; `--feature` is independent of `--config`; an `X` that does not resolve to a `FeatureDef` produces a clear message and a non-zero exit; the flag is listed in `list --help`.
