---
id: REQ-TRS-VIS-031
type: Requirement
name: A view selects what a derived IBD is about, a named style restyles it and layout hints place it, and two configurations draw as one variant view
status: verified
reqDomain: software
verificationMethod: test
---

On a derived IBD, the mapping form of `include:` **shall** select parts by `kinds` (flow kinds of
the edges at a part or its ports), `domains`, `tags` and `features` (the features its
`appliesWhen:` names, by qname or id, or a feature beneath one), keep the parts that contain a
selected part as frames, and mark every other part, port and edge faded; every writer **shall**
draw a faded element pale. `style: <name>` **shall** apply the `[diagram.style.<name>]` table of
`.syscribe.toml` over the project styles and `[diagram.style.default]`, following `extends:`;
`context = "hide"` **shall** leave the faded elements out. `layout.flow` **shall** set the layout
direction and `layout.layers` **shall** order each container's parts by band. A malformed view,
style or hint **shall** be warning `W421` and ignored; a missing or cyclic `extends` **shall** be
`W419`. `diagram export --config A --config B` **shall** draw both products in one diagram, a node
or edge only one has marked with that product's id. The list form of `include:` **shall** keep its
meaning.

**Source:** GH #338, spec §8.16.10.

**Acceptance criteria:** (a) a `kinds: [power]` view fades the signal parts and edges; (b)
`domains`, `tags` and `features` each select their parts; (c) a hiding style leaves the faded
elements out and restyles its flow kind; (d) bad styles are `W419`/`W421`, bad views `W421`; (e)
`flow`/`layers` set the direction and the band order; (f) a variant view marks what only one
product has; (g) every writer draws faded elements.
