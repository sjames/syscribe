---
id: REQ-TRS-VIS-007
type: Requirement
name: The browser lays diagrams out with ELK through sprotty-elk, honours pins, and never writes an automatic layout back implicitly
status: verified
reqDomain: software
verificationMethod: test
---

The sprotty client **shall** bind sprotty's model-layout hook (`TYPES.IModelLayoutEngine`) to
`sprotty-elk`'s `ElkLayoutEngine` backed by the `elkjs` bundled build, both vendored into the
existing esbuild bundle and served from the binary with no CDN and no runtime Node. Client-side
bounds measurement (`needsClientLayout`) **shall** be enabled so label sizes are measured before
ELK runs, and the layout **shall** place every port on its parent's border, keep sibling nodes
disjoint, keep children and labels inside their parent, and route every edge.

Pinned nodes (the `pinned` set of `REQ-TRS-VIS-006`) **shall** keep their positions: the client
**shall** pass each pinned position to ELK with interactive layering and crossing-minimisation
enabled on the root and on every compound node, so unpinned nodes are placed around them, and
**shall** use ELK's `fixed` algorithm (edge routing only; a pinned `w`/`h` winning over the
measured size) when every node is pinned.

An automatic layout result **shall never** be written to the model unless the user acts: a drag
pins the dragged node through the existing `PATCH`, *Pin all* writes every node's current
position in one `PATCH`, and *Auto-layout* clears every pin through the `DELETE` route and
re-runs ELK.

**Source:** `REQ-TRS-VIS-007` (product model).

**Acceptance criteria:** (a) `npm test` in `crates/syscribe-server/frontend/` runs the bundled
`elkjs` over a fixture IBD (a boundary with its own port, two blocks with ports, a connection and
a binding edge) with the very option values `layout.ts`'s configurator emits and passes: every
port's centre lies on its parent's border, every port label lies outside its parent, no two
siblings overlap, every child and label fits inside its parent, and both edges have sections
(`TC-TRS-VIS-007`); (b) `npm run typecheck` passes with `sprotty-elk` and `elkjs` as
dependencies of `frontend/package.json` and no CDN reference in the bundle; (c) by inspection of
`frontend/src/layout.ts`, the configurator emits `elk.position` plus the interactive options for a
partially pinned graph and `elk.algorithm: fixed` for a fully pinned one, and neither `layout.ts`
nor `container.ts` issues any write — the only `PATCH`/`DELETE` calls are the drag, *Pin all* and
*Auto-layout* handlers in `editor.ts`.
