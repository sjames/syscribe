---
id: REQ-TRS-FMED-004
type: Requirement
name: The feature model is edited in the browser through semantic operations whose effect on validity is previewed before they are written
status: verified
reqDomain: software
verificationMethod: test
---

`POST /api/feature-model/edit` **shall** accept an operation (`add`, `remove`, `rename`, `setGroup`,
`setMandatory`, `setAbstract`, `move`, `addConstraint`, `removeConstraint`, `setParameter`, `removeParameter`) on a feature, whether it lives in its own file or is an entry of a `featureTree:` sheet, apply it to the files the
feature lives in through the guarded-write engine, and return the operation that undoes it. It
**shall** return the effect of the edit on the model's validity (features that become dead or
false-optional, a model that becomes void, configurations that become invalid) and **shall not**
write an edit that makes validity worse unless it is confirmed. A rename or move **shall** rewrite
every reference to the feature including the keys of a `Configuration`'s `features:` map. The page
**shall** offer each operation, hold a worsening edit behind a confirmation that lists its
consequences, let a feature be dragged onto another to reparent it, and support undo and redo.

**Source:** `REQ-TRS-FMED-004` (product model).

**Acceptance criteria:** (a) each operation writes the right files and its undo restores the model
byte for byte; (b) an edit the engine refuses (a duplicate, a bad name, a cycle, a subtree without
the flag, a sheet entry) says why and changes nothing; (c) a preview reports the delta and writes
nothing; (d) an edit that makes a feature dead, forced, the model void or a configuration invalid is
held until `acceptWorse` and its delta is returned; (e) removing a feature removes the constraints
and configuration choices that named it; (f) moving or renaming rewrites map keys (`features:`,
`parameterBindings:`) and leaves a prefix-sharing sibling alone; (g) the history undoes and redoes in
order and a fresh edit clears the redo stack; (h) a drop lands on the smallest feature box under the
point, never on the dragged feature or its descendants; (l) a feature is marked and unmarked abstract in either layout with an exact undo, `false` is not written; products are counted and enumerated over concrete features only, a configuration must not name an abstract feature (`E238`, selected or not; no `E225` for omitting one, no `E219` for a `requires:` of one it omits), an `appliesWhen:` naming one is `W238` and an abstract feature with no children `W239`, and it stays selectable in the configurator; (i) every operation also works on a sheet entry (renaming and moving it with its subtree, keeping derived ids, following relative paths in the sheet, removing its cross-tree constraints), with an exact undo, and is refused with a reason when it would leave the sheet; (j) a parameter is added and replaced in either layout, a parameter that a configuration binds is refused naming the configurations and removed only after `removeBinding` has removed its bindings, and the impact view lists the bindings; (k) the MCP `edit_feature` tool dry-runs by default, reports the delta, holds a worsening edit until `accept_worse`, returns the undo and is hidden in read-only mode.

The MCP server **shall** offer the same operations as one tool, `edit_feature`, with the same validity delta, the same hold for an edit that makes validity worse and the undo operation in its reply.
