---
id: REQ-TRS-VIS-028
type: Requirement
name: Diagrams show what each connection carries and can place ports by flow kind
status: verified
reqDomain: software
verificationMethod: test
---

A `flowKind:` (`power`, `signal`, `command`, `data`, `protection`, `mechanical`) on a `PortDef`, a
port feature or a `connections:` entry **shall** give the edge it draws a kind: the entry's own,
else the source port's, else the target port's. Every diagram writer (SVG, sprotty, DOT, Mermaid,
PlantUML) **shall** draw the kind with its style, and a `[diagram.styles.<kind>]` table in
`.syscribe.toml` **shall** replace the built-in style of that kind. A `[diagram.portSides]` rule
**shall** place the ports of a kind on a fixed side or by direction, and a port's own `side:`
**shall** win over every rule. An unknown kind or side, or a malformed `[diagram]` entry, **shall**
be warning `W419` and be ignored. A model that names no flow kind **shall** draw as before.

**Source:** GH #335, spec §8.16.9.

**Acceptance criteria:** (a) the kind is resolved from the entry, port and `PortDef` in that order;
(b) SVG, Mermaid, DOT and PlantUML carry the kind's style; (c) a `[diagram.styles]` entry overrides
it; (d) a port-side rule places ports and `side:` wins; (e) bad values are `W419` and ignored;
(f) without flow kinds the IR, SVG and Mermaid carry no trace of the feature.
