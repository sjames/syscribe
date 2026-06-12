---
id: REQ-TRS-SPEC-002
type: Requirement
name: The discoverable spec shall include a ports & interfaces decision guide grounded in SysML v2 mechanisms
status: draft
reqDomain: software
verificationMethod: test
---

Ports and interfaces are where model authors (and LLMs) most often produce subtly wrong models — conflating `InterfaceDef` with `ConnectionDef`, mishandling conjugation/direction, or never completing the connect-two-ports wiring. The schema is documented, but the **conceptual orientation** is not. The discoverable spec **shall** include a concise *Ports & Interfaces* decision guide, grounded in the SysML v2 mechanisms, surfaced by `syscribe spec types`. It **shall** convey:

1. **The relationship** — an `InterfaceDef` *is a kind of* `ConnectionDef` whose ends are restricted to ports (SysML v2 §7.14: "an interface is simply a connection all of whose ends are ports"); a `Port` is a usage of a `PortDef`.
2. **A decision cue** — which construct to use when: a `Port` (typed by a `PortDef`) to expose an interaction point; an `InterfaceDef` for a reusable compatible pairing of two ports; a `ConnectionDef` for arbitrary feature/part connections; a connection usage (`connections:` with `from`/`to` feature chains) to wire two specific ports; `flowConnections:` to move items; `bindingConnections:` to equate features.
3. **The conjugation rule** — the receiving end is the *conjugate* of the sending end (directions flip: `in`↔`out`, `inout` is self-conjugate; SysML v2 §7.12.3), expressed via `conjugates:` on a PortDef or `isConjugated: true` on a usage/interface end.

The same guidance **shall** be present, in fuller form, in the format spec (`spec/markdown-sysml-format.md`, near §8.3) and, condensed, in the LLM authoring prompt (`prompts/create-model.md`).

**Source:** GH #24; SysML v2.0 Part 1 §7.12 (ports), §7.13 (connections), §7.14 (interfaces), §7.16 (flows).

**Acceptance criteria:** `syscribe spec types` output contains a ports/interfaces guide that states the interface-is-a-connection-of-ports relationship and the conjugation direction-flip rule, and distinguishes `Port`/`PortDef`/`InterfaceDef`/`ConnectionDef`. The format spec contains the same guide near §8.3.
