---
id: REQ-TRS-TYPE-020
type: Requirement
name: Tool shall recognise and validate IEC 62443 Zone and Conduit elements
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise two native IEC 62443 element types — **`Zone`** (id `ZN-*`) and
**`Conduit`** (id `CD-*`) — and the structural-element fields `targetSL`/`achievedSL`/`inZone`
(§13, GH #61), modelling industrial cybersecurity as security zones connected by conduits.

- A `Zone` groups `PartDef`/`Part` elements via `members:`, carries a required Security Level
  `targetSL` (1–4) and an optional assessed `achievedSL`. A `Conduit` connects two zones
  (`fromZone`/`toZone`) with an optional `achievedSL` and `implementedBy:` controls. A
  `PartDef`/`Part` may declare `targetSL`/`achievedSL`/`inZone:`.

### Validation rules

| Code | Condition |
|---|---|
| `E950` | `Zone` missing `id`, `name`, `status`, or `targetSL`. |
| `E951` | `Zone.id` does not match the `ZN-*` pattern. |
| `E952` | `Conduit` missing `id`, `name`, `status`, `fromZone`, or `toZone`. |
| `E953` | `Conduit.id` does not match the `CD-*` pattern. |
| `E954` | `Conduit.fromZone`/`toZone` unresolved or not a `Zone`. |
| `E955` | `Zone.members:` entry unresolved or not a `PartDef`/`Part`. |
| `E956` | `PartDef`/`Part.inZone:` unresolved or not a `Zone`. |
| `W950` | `Zone.achievedSL < targetSL` (security level not yet achieved). |
| `W951` | `Conduit.achievedSL` below either connected zone's `targetSL` (opt-in). |
| `W952` | A `PartDef`/`Part` declares `targetSL` but belongs to no zone (opt-in). |
| `W953` | An `approved` `Zone` with `targetSL >= 2` is referenced by no `Conduit`. |

### CLI

`zones [--coverage] [--json]` (list zones + SL gap, or Zone × SecurityControl cross-table),
`conduits [--json]` (list conduits + SL adequacy), and `template Zone` / `template Conduit`.

**Source:** §13 (IEC 62443 Zone/Conduit Model), GH #61.

**Acceptance criteria:**

- `Zone` and `Conduit` parse with all required fields; `E950`–`E956` fire on the matching
  defects.
- `W950` fires when `achievedSL < targetSL`; `W953` fires for an approved leveled zone with
  no conduit.
- `zones` and `conduits` produce correct output; `template Zone`/`template Conduit` produce
  valid skeletons.
- The demo model is extended with Zone/Conduit examples and validates clean.
