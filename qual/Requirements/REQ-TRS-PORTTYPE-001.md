---
id: REQ-TRS-PORTTYPE-001
type: Requirement
name: Tool shall check a machine's events and payloads against the PortDef items of the port they travel through
status: draft
reqDomain: software
verificationMethod: test
---

A port may be typed by a `PortDef` (`typedBy:`, optionally `isConjugated: true`) whose `features:` typed by an `ItemDef` are the events it carries, each with an optional `direction:`; the `ItemDef`'s `features:` are the payload fields (spec §8.8.5). The same `PortDef` can then type the architecture port and the machine's port.

The tool **shall**, for a part that exhibits a state machine:

- report a `send`/`accept` through a typed port of an event the `PortDef` does not carry as `E932`, naming the events it carries;
- report a `send`/`accept` of an item whose direction at the port (reversed when the port is conjugated) does not allow it as `E932`;
- report a `send` payload key, or a reference `m.f` to an accepted event bound `as: m`, naming a field the item's `ItemDef` does not have as `E932`;
- report a `send` payload value, or an expression using an accepted field, whose type or unit dimension cannot match the field as `E935`, comparing units by dimension;
- keep the checks of an untyped port unchanged;
- when a part `models:` an architecture element, take a port typed by the same `PortDef` and conjugation as exactly one port of that element as its counterpart without `portMap:`, and report corresponding ports typed differently as `E991`.

**Source:** GH #331, found while building the grid-tie inverter model (`model_gti/`).

**Acceptance criteria:**

- A sender and a conjugated receiver typed by one `PortDef` validate clean.
- Each mismatch kind gives its code.
