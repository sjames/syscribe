---
id: REQ-TRS-FMEA-004
type: Requirement
name: "An FMEASheet row that cannot become an FMEAEntry, or whose explicit rpn disagrees with S×O×D, shall be reported rather than silently altered"
status: draft
reqDomain: software
verificationMethod: test
---

The walker explodes each `FMEASheet` `entries:` row into an `FMEAEntry` element keyed by its `id:`. A row that cannot be exploded **shall not** disappear silently from a safety analysis:

- An `entries:` row that has no string `id:` (or is not a mapping at all) **shall** raise error **`E923`** on the `FMEASheet` file. The message **shall** name the row's 1-based position in `entries:` and, where present, its `failureMode:`/`name:` so the author can find it. The row is still not exploded (it has no identity to key an element by).
- When a row declares all three of `fmeaSeverity:` (or `severity:`), `occurrence:` and `detection:` **and** an explicit `rpn:` whose value differs from their product, the tool **shall** keep the computed `S × O × D` as the entry's RPN and **shall** raise warning **`W928`** on the `FMEASheet` file naming the row id, the explicit value, and the computed value. An explicit `rpn:` that equals the product, or an explicit `rpn:` on a row missing any of the three factors (where it is the only RPN available), raises nothing.

**Source:** GH #132 — a row without `id:` vanished from `validate` and `fmea report` with no finding (the walker's comment claimed the validator warned; it did not), and an explicit `rpn:` was silently replaced by `S×O×D`.

**Acceptance criteria:** an `FMEASheet` with a row lacking `id:` raises `E923` naming the row's position and failure mode; a row with `S=5, O=4, D=3, rpn: 100` raises `W928` naming `100` and `60`, and `fmea report` shows RPN `60` for it; a row with `rpn:` equal to `S×O×D` raises no `W928`; a row with `rpn:` but no `detection:` raises no `W928`.
