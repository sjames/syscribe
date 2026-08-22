---
type: Requirement
id: REQ-TRS-SYSMLV2-024
name: "A SysMLv2 flow def/flow maps to the native FlowDef/Flow schema; a nested flow usage's endpoints also lift onto the owning part's flowConnections:"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-007]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - flows
---

A `flow def` shall be synthesized into a native `FlowDef` element carrying `supertype:`/`doc`. A
*named* `flow`/`message`/`succession flow` usage shall be synthesized into a native `Flow` element
carrying `itemType:`/`doc`. Additionally, every `FlowUsage` (named or anonymous) found directly
inside a `part def`/`part` usage body shall lift its `from:`/`to:`/`kind:`/`item:` onto the *owning*
part's own `flowConnections:` field — the exact dual pattern `REQ-TRS-SYSMLV2-010` already
established for `connect` statements and `connections:`.

## Rationale

`ElementType::FlowDef`/`ElementType::Flow` already existed in the native schema, exercised by two
real hand-authored files (`model/Flows/{PowerFlowDef,TelemetryFlowDef}.md`), but were unreachable
from SysMLv2 ingestion. Unlike Concern, `FlowDef`/`FlowUsage` are already reachable from all three
dispatch enums this module cares about (`PackageBodyElement`, `PartDefBodyElement`,
`PartUsageBodyElement`) — no parser-level ceiling blocks the base mapping. The `flowConnections:`
lift closes the same gap `REQ-TRS-SYSMLV2-010` closed for plain connections, applied to the sibling
relationship kind the AST happens to model almost identically.

## Scope

- `FlowDef`/`FlowUsage` are two distinct AST structs (unlike `ConcernUsage`'s single-struct/
  `is_definition`-flag design) — two separate conversion functions, `convert_flow_def`/
  `convert_flow_usage`, mirroring the `View(Def/Usage)` precedent's shape more than Concern's.
- `FlowDef.body`/`FlowUsage.body` share a deliberately thin `DefinitionBody`
  (`Semicolon | Brace { elements: Vec<DefinitionBodyElement> }`, `DefinitionBodyElement` only:
  `Error`/`Doc`/`OccurrenceMember`/`Other`) — also shared by `AllocationDef`/`AllocationUsage`/
  `OccurrenceDef`. `ends:`/`itemType:` (§8.6.1, the shape `model/Flows/PowerFlowDef.md` uses on a
  `FlowDef`) are **not** derived from this body: real fixtures show a `flow def` body *can* contain
  nested `attribute`/`part`/`flow` members (reachable only via the generic
  `OccurrenceMember(OccurrenceBodyElement)` variant), but nothing in the AST unambiguously marks a
  member as "this is an end port" the way a nested `StateUsage`/`ActionUsage` child was unambiguous
  for State/Action — an explicit descope, not an oversight.
- `doc /* ... */` inside a `flow def`/`flow` usage body is **not** a direct
  `DefinitionBodyElement::Doc` the way every other body type in this module lifts doc text —
  confirmed empirically (parsing real source and inspecting the AST, not assumed from the enum
  shape) — it lands wrapped as `OccurrenceMember(OccurrenceBodyElement::Doc)` instead. Both shapes
  are checked; every other `OccurrenceBodyElement` variant stays unwalked per the point above.
- `item_type:` (the existing native `RawFrontmatter.item_type`, §8.6.1) is populated from
  `FlowUsage.payload.type_name` (the `of` clause) or `FlowUsage.type_name` (the bare `:` typing
  shorthand) — both item-shaped per real parser fixtures showing the two forms as parallel,
  interchangeable ways to identify *what flows*, matching Syscribe's own spec framing of `itemType`
  as "shorthand: qualified name of the ItemDef carried by this flow". **Never** `typedBy:` — there is
  no AST field distinct from the item-type source that would represent "typed by an actual FlowDef".
  `payload.multiplicity` (the `of name : Type[mult]` cardinality) has no existing multiplicity-to-
  string renderer anywhere in this module and is out of scope, the same class of descope as Concern's
  `requires:`/`assume:`.
- `flowConnections:` entries (`{name?, from, to, kind, item?}`, §8.6.2) are built for *every*
  `FlowUsage` found directly in a `part def`/`part` usage body — named or anonymous alike, mirroring
  `part_def_connection_entries`'s identical "regardless of name" posture. `kind:` uses the exact
  §8.6.2 vocabulary (`Flow → streaming`, `Message → message`, `SuccessionFlow → succession`).
  `from`/`to` reuse `qualify_connection_end`'s existing sibling-lookahead/truncation logic
  unchanged, including `REQ-TRS-SYSMLV2-015`'s `W542` truncation warning.
- **A real, empirically-discovered AST fact, not part of the original plan**: `FlowUsage.from`/`.to`
  are typed as a general `Expression` (the value-expression grammar's postfix `.` chaining), *not*
  the dedicated `path_expression` production `connect` endpoints use — so a dotted flow endpoint
  (`a.x`) parses as nested `Expression::MemberAccess`, never `Expression::FeatureChainRef`.
  `connection_end_display` (the shared endpoint-to-string helper `REQ-TRS-SYSMLV2-010` already
  established) gained a new `MemberAccess` arm to handle this — confirmed, by direct testing, not to
  change `connect` endpoint behavior at all (those never produce `MemberAccess`).
- A `FlowUsage` nested inside an `ActionDef`/`ActionUsage` body is *already*, separately, excluded by
  `REQ-TRS-SYSMLV2-019`'s own action-body walker — unaffected by this requirement.
  `InterfaceDefBodyElement`/`OccurrenceBodyElement`/`RequirementDefBodyElement` also carry a
  `FlowUsage` variant per the AST, but none of those bodies are recursively walked for nested-element
  extraction anywhere in this module — this requirement's scope is exactly `PackageBodyElement`/
  `PartDefBodyElement`/`PartUsageBodyElement`, matching every other mapped kind.

**Acceptance criteria:** a package-wrapped `flow def` synthesizes a real `FlowDef` with `supertype:`
set and `ends:`/`item_type:` absent; a named top-level `flow t : Fuel from a to b;` synthesizes a
real `Flow` with `itemType: Fuel`; an anonymous `flow a.x to b.y;` nested in a `part def` body
contributes a `flowConnections:` entry on the owning part with no separate element; a *named* nested
flow usage produces **both** a standalone `Flow` element **and** a `flowConnections:` entry;
`message`/`succession flow` usages lift `kind: message`/`kind: succession`; a genuinely
two-segment, non-redeclared flow endpoint raises `W542` exactly as a connection endpoint would.
