---
type: Requirement
id: REQ-TRS-PUML-026
name: "Shape display names resolved from referenced element name field"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PUML-001]
breakdownAdr: Decisions::PlantUMLADR
tags:
  - diagram
  - plantuml
---

When generating PlantUML source, the display name for each shape shall be resolved from
the referenced element's `name` field in the model. When the referenced element is not
found in the loaded element set (e.g. external reference), the display name falls back
to the last `::` segment of the `ref` qualified name. The PlantUML node alias (used in
edge references) shall always be the shape's key in the `shapes:` map (e.g.
`s-uavsystem`), sanitised to a valid PlantUML identifier by replacing `-` with `_`.

## Sanitisation rule

The shape key is sanitised by replacing every `-` character with `_`. No other
substitutions are applied. If after sanitisation the alias begins with a digit, a leading
`s_` prefix is prepended to ensure the alias is a valid PlantUML identifier.

## Example

| Shape key | Sanitised alias |
|---|---|
| `s-uavsystem` | `s_uavsystem` |
| `s-fc-primary` | `s_fc_primary` |
| `42-node` | `s_42_node` |

## Rationale

Using the shape key as the alias rather than a derived-from-name alias avoids collisions
when two shapes reference elements with the same `name` but different qualified names.
The display name (quoted in the PlantUML `class "Name" as alias` syntax) carries the
human-readable label; the alias carries the stable structural identity within the diagram.
