---
id: REQ-TRS-TYPE-004
type: Requirement
name: "Tool shall recognise and validate the EventOccurrenceDef and EventOccurrence elements"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `EventOccurrenceDef` definition element and its `EventOccurrence` usage.

- **SysMLv2 mapping:** `EventOccurrenceDef` maps to `event occurrence def` and `EventOccurrence` to `event occurrence` (§2.1, §2.2). An `EventOccurrenceDef` classifies momentary, instantaneous occurrences (no duration). An `EventOccurrence` usage models a momentary observation or signal emission, distinguished by `direction:` (`in` = observed, `out` = emitted).
- **Identity style:** both are **name-based (Style B)** — identity is the path-derived qualified name (basic-name grammar), with `supertype:` for specialization (EventOccurrenceDef) and `typedBy:` for typing (EventOccurrence). They carry **no stable id**.
- **Required vs optional fields:** `type:` is required; `name:` is optional and defaults to the filename stem. `direction:` on an `EventOccurrence` is optional.
- **Recognition behaviour:** a file with `type: EventOccurrenceDef` or `type: EventOccurrence` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]). An `EventOccurrenceDef` with no explicit `supertype:` receives the implicit base-library supertype `Occurrences::EventOccurrence` per [[REQ-TRS-ELEM-003]] (§11.4).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-ELEM-003]].

**Source:** §2.1, §2.2, §11.4

**Acceptance criteria:** A model containing a minimal `EventOccurrenceDef` and an `EventOccurrence` typed by it parses both at their declared types (visible in `export`) and raises no `E005` for either; a sibling file with an unrecognised `type:` value raises `E005`.
