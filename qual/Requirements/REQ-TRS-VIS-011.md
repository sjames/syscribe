---
id: REQ-TRS-VIS-011
type: Requirement
name: The browser offers Pin all, Auto-layout and Save companion SVG, each a single guarded write
status: verified
reqDomain: software
verificationMethod: test
---

The diagram panel **shall** offer three actions beside *Add*, *Connect* and *Delete*:

- **Pin all** — writes every placed node's current position (and size, when ELK sized it) as
  pins in one `PATCH /api/diagrams/layout/{qname}`; the server **shall** write `x`, `y`, `w` and
  `h` when given and **shall** keep an existing `w`/`h` when a later patch carries only `x`/`y`;
  a `null` value **shall** remove that one pin and leave the others;
- **Auto-layout** — clears every pin with `DELETE /api/diagrams/layout/{qname}`, which **shall**
  drop the diagram's whole `layout:` key and nothing else, then re-runs ELK; an unknown qname or
  a non-`Diagram` target **shall** be refused with a reason;
- **Save companion SVG** — serialises the current render as an SVG conforming to spec §8.16.5
  and sends it as `{ "svg": … }` to `PUT /api/diagrams/svg/{qname}`; the server **shall** write
  it to the diagram's `svgFile:` (default `./<stem>.svg` beside the `.md`, an existing
  `svgFile:` honoured as is), set `svgMode: companion` and `svgFile:` when absent, append one
  `<img>` to the body and never a second on a later save, and **shall** refuse a body that is not
  an SVG document or a non-`Diagram` target without writing anything.

Each action **shall** be a guarded write that returns the `WriteResponse` delta and reports a
refusal, leaving the browser's picture as it was.

**Source:** `REQ-TRS-VIS-011` (product model).

**Acceptance criteria:** through the real router, (a) a `{x, y, w, h}` patch writes all four and
an `{x, y}` patch afterwards keeps `w`/`h`; (b) a `null` patch on a two-pin diagram removes one
pin and keeps the other, and unpinning a shape with no pin is a harmless committed no-op;
(c) `DELETE` removes the `layout:` key, keeps the shapes and body, leaves `pinned` empty on the
next `GET`, and refuses an unknown qname and a `PartDef`; (d) a `PUT` with a valid SVG writes the
file, sets `svgMode`/`svgFile`, appends the `<img>` once, raises no `E402`/`W405`, and a second
`PUT` overwrites without appending; (e) an explicit `svgFile: pictures/custom.svg` is written
relative to the `.md` and left unchanged; (f) `<div>`, an unterminated `<svg`, an empty string
and plain text are each refused with a reason naming SVG and the `.md` is byte-identical
afterwards (`TC-TRS-VIS-011`).
