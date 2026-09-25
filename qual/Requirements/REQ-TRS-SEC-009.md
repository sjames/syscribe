---
id: REQ-TRS-SEC-009
type: Requirement
name: Tool shall roll up an attack tree's feasibility from its structural root node, independent of file order
status: draft
reqDomain: software
verificationMethod: test
---

`REQ-TRS-SEC-003` defines an `AttackTree`'s computed feasibility as "the value
of its single root child". The tool **shall** identify that root structurally:
the root is the one `AttackTreeGate` or `AttackStep` under the tree's
qualified-name prefix that **no** other gate of the same tree lists in its
`inputs:` (a gate listing itself does not count). The root **shall not** depend
on directory or file order — a sub-gate that sorts before the root gate is not
the root.

When no such node exists (every node is some gate's input — a cycle) or more
than one exists (a forest of disconnected sub-trees), the tree has no unique
root and its feasibility is **not computable**: `W035` is not evaluated for it,
exactly as for any other uncomputable roll-up.

**Source:** GH issue #149.

**Acceptance criteria:** an attack tree whose root OR gate sorts after its AND
sub-gate rolls up from the OR gate, so a `ThreatScenario` declaring the OR
gate's value raises no `W035`, and a `ThreatScenario` declaring a different
value raises a `W035` naming the root's computed value; reversing the element
order does not change the computed feasibility.
