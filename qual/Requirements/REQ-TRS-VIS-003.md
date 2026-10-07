---
id: REQ-TRS-VIS-003
type: Requirement
name: A Diagram with a subject and no shapes is derived from the model, with include/exclude filters, deterministic shape ids and W417/W418
status: verified
reqDomain: software
verificationMethod: test
---

A `Diagram` element that declares a `subject:` and no `shapes:` block **shall** have its IR
derived from the model by the generator registered for its `diagramKind` (`BDD` and `IBD`
first; any other kind yields an empty graph until its generator ships). A `Diagram` that
declares `shapes:` **shall** be manifest-sourced. No mode field is introduced: the presence of
`shapes:` selects the source.

A derived diagram may declare `include:` (restrict the content to the named members of the
subject and the edges joining them) and `exclude:` (remove named members and every edge
touching them), each a string or a list of qualified names or names relative to the subject.
An entry naming no member of the subject **shall** raise warning `W417`; `include:`/`exclude:`
on a manifest diagram **shall** raise one `W417` and be ignored, the manifest drawn as listed.
A `subject:` whose type is not valid for the diagram's kind under spec §8.16.8 **shall** raise
warning `W418` and the diagram **shall** be drawn empty.

Derived shape ids **shall** be deterministic functions of the depicted element's qualified name
(`s-` followed by the name lower-cased with `::` and every non-alphanumeric run replaced by
`-`), so `layout:` pins keyed by shape id apply to a derived diagram unchanged and survive
regeneration. The generated nodes **shall not** provoke the manifest-reference warnings
`W402`/`W403`.

**Source:** `REQ-TRS-VIS-003` (product model).

**Acceptance criteria:** through the real walker and validator on a temp-built fixture model,
(a) a `BDD` with `subject: Sys` and no `shapes:` builds an IR with six blocks, two inheritance,
three composition and one association edge matching its golden JSON snapshot, with no `W417`,
`W418` or `E405`; (b) an `IBD` with `subject: Sys::PowerSystem` builds one boundary, three
blocks, four ports, a connection and a binding edge matching its golden snapshot; (c) a
`layout:` pin keyed `s-sys-powersystem-engine` is applied and raises no `W416`; (d)
`include: [Engine, Motor, Ghost]` with `exclude: [Motor]` leaves only `Sys::Engine` and raises
exactly one `W417` naming `Ghost`; (e) `include:` on a diagram that also has `shapes:` leaves
the manifest unfiltered and raises one `W417`; (f) an `IBD` whose subject is a `Package` draws
nothing and raises one `W418` naming the type; (g) a derived `IBD` raises no `W402`/`W403` for
its generated port references. Both codes are listed in `prompts/spec/validation.md` and
`docs/validation/rules.md` (the catalogue tests enforce it).
