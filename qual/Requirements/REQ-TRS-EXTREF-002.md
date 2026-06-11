---
id: REQ-TRS-EXTREF-002
type: Requirement
title: Tool shall look up elements by extRef via a dedicated command and surface the field
status: draft
reqDomain: software
verificationMethod: test
---

So that the external correspondence introduced by [[REQ-TRS-EXTREF-001]] is usable for navigation — *"given this DNG/SysML-tool reference, which model element represents it?"* — the tool **shall** make `extRef` looked-up and discoverable.

## `extref` lookup command

The tool **shall** provide a dedicated subcommand:

```
syscribe -m <root> extref <ref> [--json]
```

- It **shall** print **every** element whose `extRef` (string or any entry of the list) is **exactly equal** to `<ref>`. Matching is on the whole reference value, not a substring or pattern.
- For each match it **shall** report at least the element's qualified name and `type:` (and stable `id:` when the element has one), so the element can be opened or cross-referenced.
- When no element declares `<ref>`, it **shall** report no match and exit non-zero (a lookup miss), leaving model cross-reference resolution (`show`, `find`, `resolve_ref`) **unchanged** — this is a separate, external-reference lookup path.
- `--json` **shall** emit the matches as a machine-readable array.

A duplicate reference ([[REQ-TRS-EXTREF-001]], `W028`) **shall** return **all** matching elements rather than an arbitrary one.

## Discoverability

The tool **shall** also:

- surface `extRef` in the element detail produced by `show <element>` — an element that declares `extRef` **shall** list its external reference(s); and
- list `extRef` in `syscribe spec fields` as a recognised common frontmatter field applicable to all element types; and
- document the field and the `extref` command in the docs (the format spec common-fields section and the CLI reference).

**Source:** lookup/discoverability companion to [[REQ-TRS-EXTREF-001]]; modelled on [[REQ-TRS-IMPL-002]].

**Acceptance criteria:** `extref <ref>` prints the element that declares `<ref>` and exits `0`; with two elements sharing `<ref>` it prints both; for an unknown `<ref>` it prints no match and exits non-zero; `extref <ref> --json` emits an array of the matches; `show <element>` on an element with `extRef` displays the reference(s); `spec fields` lists `extRef`.
