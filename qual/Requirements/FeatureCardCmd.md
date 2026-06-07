---
id: REQ-TRS-DISC-002
type: Requirement
name: feature Command (Feature Card)
title: Tool shall provide a feature <qname> command that prints a single feature's card
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a `feature <qname|name>` subcommand that prints a single feature's "card" — a focused, all-in-one view of one `FeatureDef`. The argument **shall** be accepted as either the feature's qualified name (`Features::Payload::Survey`) or its `name`.

The card **shall** show:

- the feature's documentation body (the Markdown `doc`);
- its parent / group and `groupKind`;
- its `requires:` / `excludes:` cross-tree constraints;
- its typed `parameters:` (name, type, and `range`/`enumValues` if present);
- the `Configuration` elements that select it (`true`);
- every model element gated by the feature — i.e. every element whose `appliesWhen:` expression names this feature as an operand — each listed with its qualified name and element type.

The command **shall**:

- support a `--json` flag emitting the card as structured data carrying the same fields;
- exit non-zero with a clear message if the argument does not resolve to a `FeatureDef` (unknown name, or a name that resolves to a non-`FeatureDef` element).

The command **shall** be discoverable in `--help`.

## Rationale

When reviewing or changing a single feature, an engineer needs to know its constraints, its parameters, where it ships (which configurations select it), and what it gates (which elements are conditioned on it). [[REQ-TRS-DISC-001]] gives the whole-model overview; this command gives the per-feature drill-down, including the reverse `appliesWhen:` index that is otherwise invisible.

**Source:** §9 (PLE); product-line feature discoverability; per-feature companion to [[REQ-TRS-DISC-001]].

**Acceptance criteria:** `feature Features::Payload::Survey` (or by `name`) prints the feature's doc body, parent/group and `groupKind`, `requires:`/`excludes:`, typed `parameters:`, the configurations that select it, and every element whose `appliesWhen:` names it (with qualified name and type); `--json` emits the same data; an argument that does not resolve to a `FeatureDef` produces a clear message and a non-zero exit; the command is listed in `--help`.
