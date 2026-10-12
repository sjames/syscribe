---
id: REQ-TRS-VIS-030
type: Requirement
name: A context composite marks external actors, diagrams draw them as actors and every boundary port of the system is connected
status: verified
reqDomain: software
verificationMethod: test
---

A part usage marked `role: external` **shall** be accepted as an actor outside the product and,
like `role: environment`, **shall** not count as evidence or as an architecture counterpart. A
derived IBD **shall** draw an external usage with the stereotype «actor» and an environment usage
«environment», dashed; a derived BDD **shall** draw the definition of an external usage «actor».
In a part holding at least one external usage, every port of a product part's definition that no
connection, flow or binding of that part reaches **shall** be warning `W996`. A part with no
external usage **shall** not be checked. Any other `role:` value **shall** remain `E992`.

**Source:** GH #337, spec §12.13.

**Acceptance criteria:** (a) the context IBD draws the actors «actor», the environment part
dashed, beside the system, joined to its ports; (b) the context BDD marks the actor definition;
(c) an unconnected system port is `W996` and a connected one is not; (d) without an actor there is
no `W996`; (e) `role: external` is not `E992`, an unknown role is.
