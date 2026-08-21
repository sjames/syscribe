---
type: Package
name: Legacy
foreignFormat: toydsl
---

A subsystem authored entirely in a made-up "toy DSL" (`.toy` files) instead
of Markdown+YAML. Its whole subtree is handed to the `toydsl` plugin
(`../plugin.py`, declared in `.syscribe.toml`), whose output — real
`PartDef`/`RequirementDef` elements — is merged into this graph exactly like
a hand-authored `.md` file would be.
