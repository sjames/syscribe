---
id: REQ-TRS-VIS-029
type: Requirement
name: A derived IBD nests parts, places deployed software in its hardware, marks safety and optional parts and frames parts by domain
status: verified
reqDomain: software
verificationMethod: test
---

A derived IBD **shall** draw part usages `depth:` levels deep (default 1), each nested usage inside
its owner with its ports and the connections its definition declares. A drawn deployment package
(`isDeploymentPackage: true`) allocated to hardware in any §12.9 form **shall** be drawn inside a
drawn usage of that hardware, else inside the innermost drawn usage containing it. A usage whose
element or definition declares a safety level **shall** carry a mark naming the level with a heavy
outline, and one with `appliesWhen:` a mark badged `optional`. `groupBy: domain` **shall** frame
the subject's parts by `domain:`, with containment winning over the frame; any other value is
warning `W420`. A definition that contains itself **shall** be opened once on a path, and `depth:` **shall** be capped at 16. Among several hosts a deployment package **shall** go into the nearest. Parents **shall** precede their children in the node order. Without these inputs
the derived IBD **shall** be unchanged.

**Source:** GH #336, spec §8.16.8.2.

**Acceptance criteria:** (a) firmware allocated to an MCU is drawn inside the MCU inside its board at
depth 2, and inside the board at depth 1; (b) a SIL 2 part is outlined and badged, an `appliesWhen`
part badged optional; (c) `groupBy: domain` frames parts, a deployed package keeps its place and
shows its domain as a banner; (d) an unknown `groupBy` is `W420`; (e) an edge joins ports across
levels; (f) a model with none of these draws the same part tree as before.
