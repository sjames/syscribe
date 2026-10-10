---
type: Requirement
id: REQ-TRS-TMPL-001
name: "template Requirement uses the model's configured id prefix"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - ids
---

When the model declares additional requirement id prefixes in `[ids.prefixes]`, `template Requirement` shall
print an `id:` that uses a configured prefix instead of the literal `REQ-PREFIX-001`.

## Behavior

- `template Requirement --prefix <P>` prints `id: <P>-<CATEGORY>-001`'s skeleton with `<P>` as the leading prefix. An unconfigured `<P>` is
  rejected with the list of valid prefixes (built-in `REQ` plus configured ones).
- Without `--prefix`, a model that configures `Requirement` prefixes defaults to the first configured prefix; a model with none keeps `REQ`.
- Other element types are unaffected. The MCP `template` tool returns the same text.
