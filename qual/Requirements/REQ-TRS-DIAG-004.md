---
id: REQ-TRS-DIAG-004
type: Requirement
name: Diagram commands shall treat an unresolvable element as a not-found error
status: draft
reqDomain: software
verificationMethod: test
---

Every `diagram` subcommand that is asked to draw or measure a named model
element **shall** fail when a named element does not exist, consistent with
the other commands' not-found behaviour: print `error: element '<qname>' not
found` on stderr for each unresolvable qualified name, write nothing to
stdout, and exit `1`. It **shall never** silently skip the element or
substitute a placeholder-sized box for it. This covers:

- `diagram measure <qnames>` — any comma-separated qname that does not resolve;
- `diagram compose <layout.json>` and `diagram layout <placement.json>` — any
  placed element (`elements[].qname`) that does not resolve;
- `diagram compose <DiagramQName>` — any `expose:` entry of the Diagram that
  does not resolve (a missing Diagram element itself was already an error);
- `diagram render`, `diagram seq` and `diagram req` — an unknown element
  (already an error; unchanged).

When every named element resolves the commands behave as before.

**Source:** GH issue #168 — `diagram measure "Nope::X"` printed a warning,
emitted `[]` and exited 0; `compose` skipped unknown placements and `layout`
drew them at a default size, also exiting 0.

**Acceptance criteria:** `diagram measure Nope::X` exits 1 with `error:
element 'Nope::X' not found` on stderr and empty stdout; a measure list mixing
a real and an unknown qname exits 1 naming only the unknown one; `diagram
measure <real>` exits 0 with JSON naming it; `diagram layout` and `diagram
compose` of a placement/layout file naming an unknown element exit 1 naming
it.
