---
type: Requirement
id: REQ-TRS-VAL-017
name: "W600 shall be suppressed on a Part usage whose typedBy: target already carries non-empty documentation"
status: draft
reqDomain: software
reqClass: system
verificationMethod: test
---

`W600` ("PartDef/Part has an empty documentation body") shall not fire on a `type: Part` element
with an empty `doc` body when its `typedBy:` resolves to another element with a non-empty `doc`
body. `W600` shall continue to fire exactly as today for: a `PartDef` itself with an empty `doc`
(regardless of anything else); a `Part` usage whose `typedBy:` target also has an empty `doc`; and
a `Part` usage whose `typedBy:` doesn't resolve to any known element.

## Rationale

A deeply-composed model — `part x : Services::Foo;`, referencing a fully-documented `Services::Foo`
`PartDef` — is not actually missing documentation; it's one lookup away. `W600` as originally
specified doesn't distinguish this from a genuinely undocumented usage, producing noise
proportional to how deeply a model composes rather than to how much documentation is actually
missing. This is a general validator characteristic (native Markdown and SysMLv2-synthesized
content are affected identically) surfaced in practice by a `sysmlSubmodel: true` subtree's
composition-heavy authoring style, where a bare `part x : Type;` reference to an already-documented
type is the norm rather than the exception.

## Scope

- Applies uniformly to every `type: Part` element regardless of origin — no SysMLv2-specific
  branching, no new field, no new configuration.
- A `PartDef` itself is unaffected — `W600` on a `PartDef` with an empty `doc` fires regardless of
  anything else, since a `PartDef` is the type being referenced, not a usage of one; there is
  nothing further to fall back to.
- The `typedBy:` lookup reuses the existing `Resolver` unchanged — no new resolution logic, and no
  new finding when `typedBy:` fails to resolve (that's already a separate, existing concern; this
  requirement's suppression simply doesn't apply in that case, `W600` still fires).
- Does not change `W601` (`ActionDef`/`Action` documentation) or any other empty-documentation
  check — scoped to `W600` specifically.

**Acceptance criteria:** `part x : SomeDocumentedPartDef;` with an empty own `doc` does not trigger
`W600` when `SomeDocumentedPartDef.doc` is non-empty; `part x : SomeUndocumentedPartDef;` still
triggers `W600`; a `PartDef` itself with an empty `doc` still triggers `W600` regardless of
anything else; a `Part` usage whose `typedBy:` doesn't resolve still triggers `W600`.
