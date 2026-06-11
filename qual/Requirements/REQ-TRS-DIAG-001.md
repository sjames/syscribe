---
id: REQ-TRS-DIAG-001
type: Requirement
title: Tool shall enforce all Diagram element validation rules E400–E402 and W400–W412
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce every Diagram-element validation rule in the following table, as defined in §11.12 of the Syscribe format specification. Each rule **shall** be emitted when the relevant condition is detected during model validation.

| Code | Condition |
|---|---|
| `W400` | `type: Diagram` element has no `diagramKind` field and `svgMode` is not `companion` |
| `E400` | `diagramKind: Mermaid` but the body contains no ` ```mermaid ` fenced block |
| `E401` | `diagramKind: PlantUML` but the body contains no ` ```plantuml ` fenced block |
| `E402` | `svgMode: companion` but the companion SVG file does not exist on disk, or `svgFile:` is set but the referenced file does not exist |
| `W401` | `subject:` field does not resolve to a known element |
| `W402` | A shape in `shapes:` has a `ref:` that does not resolve to a known element (and has no resolvable ancestor) |
| `W403` | An edge in `edges:` has a `source` or `target` that does not reference a shape id defined in `shapes:` |
| `W405` | `svgMode: companion` body has no `<img` tag, or `svgMode: inline` body has no ` ```svg ` fenced block |
| `W406` | A frontmatter `shapes:`/`edges:` id has no matching `id` attribute in the inline SVG body |
| `W407` | An inline SVG `id="…"` attribute has no matching entry in frontmatter `shapes:`/`edges:` |
| `W408` | A Mermaid `%% ref:` annotation does not resolve to a known element |
| `W409` | A Mermaid diagram has no `%% ref:` annotations at all |
| `W410` | A Mermaid `%% link:` annotation does not resolve to a known element |
| `W411` | A shape `link:` value does not resolve to a known element |
| `W412` | A relative `href="…"` in an inline SVG body does not match any model element file |

**Source:** §11.12 (Diagram validation rules)

**Acceptance criteria:** For each code, a crafted model fixture that triggers exactly that condition produces at least one finding with that code.
