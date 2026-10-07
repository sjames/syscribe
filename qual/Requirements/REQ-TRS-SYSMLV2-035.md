---
id: REQ-TRS-SYSMLV2-035
type: Requirement
name: Tool shall map a SysMLv2 use case def/use case onto a native UseCaseDef/UseCase carrying supertype/typedBy, subject, actors, objectives, isAbstract and doc
status: verified
reqDomain: software
verificationMethod: test
---

A `use case def` shall be synthesized into a native `UseCaseDef` and a named `use case` usage
into a native `UseCase`, at package level and nested in a `part def` body. The body is the same
`UseCaseDefBody` the case family already uses (`REQ-TRS-SYSMLV2-026`), so the same lift applies:
`subject:`, `actors:`, `objectives:`, the first `return` type as `result:`, `isAbstract:` and `doc`;
plus `supertype:` (def) or `typedBy:` (usage).

`include`/`extend` statements are not lifted (their AST carries a usage name, not the qualified
target the native `includes:`/`extends:` fields require); `refines:` has no SysMLv2 source. A
non-`draft` ingested `UseCaseDef` therefore raises the existing advisory `W307` (no `refines:`),
like any hand-authored one. Mapped use cases no longer count toward `W543`.
