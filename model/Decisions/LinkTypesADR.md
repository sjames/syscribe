---
type: ADR
id: ADR-SYS-LINKTYPE-001
name: "User-defined link types: declared in .syscribe.toml, authored under a links: map, optionally extending a built-in trace link with per-type rule relaxation"
status: accepted
tags:
  - traceability
  - link-types
---

## Context

Every relationship Syscribe understands today is hard-coded: a dedicated frontmatter field
(`satisfies:`, `verifies:`, `derivedFrom:`, `refines:`, …), a variant of `graph::EdgeKind`, and a
set of validator rules bound to that field (`E104`, `E105`, `E310`, `E312`, `E313`, `E316`, the
coverage warnings `W002`/`W300`/`W305`, …). Projects routinely need relationships the
format does not name — `mitigates` (control → hazard), `conflictsWith` (requirement ↔
requirement), `partiallySatisfies`, `informs`, `supersedesDesignOf` — and today they either abuse
a built-in field (inheriting rules that do not fit, e.g. `E313` domain matching or `E312`
"no parent assignment"), or push the relationship into `custom_fields`, where nothing resolves,
validates, or traverses it.

Two needs pull in different directions:

1. A **free-standing** relationship with its own, declared constraints (legal source/target
   types, multiplicity, acyclicity), unrelated to any built-in rule.
2. A **variant of a built-in** trace link — "this is a `satisfies`, but a partial one that must
   not trip `E312`/`E313`" — which should keep every other `satisfies` semantic (the reverse
   index, the coverage rules, impact traversal) while relaxing named rules for that variant only.

## Decision

- **Declaration lives in `.syscribe.toml`**, in a `[linkTypes.<name>]` table — the same
  config-gated, opt-in posture as `[ids.prefixes]`, `[users]`, `[repos]`. A model without the
  table behaves exactly as before. A link-type name is lowerCamel (`^[a-z][A-Za-z0-9]*$`) and
  must not collide with a built-in link/field/reverse-index name, another declared type, or
  another type's `inverse`. Keys: `description`, `inverse`, `sourceTypes`, `targetTypes`,
  `cardinality` (`N`, `N..M`, `N..*`), `acyclic`, `suspect`, `extends`, `relax`, `coverage`.
  A malformed entry is `W630` and ignored as a whole (so its uses then surface as `E630`); an
  unknown key is `W630` and otherwise harmless.
- **Instances are authored under one namespaced field**, `links:`, a map from link-type name to
  a reference or list of references (id or qname, resolved exactly like `satisfies:`). Keeping
  user-defined names out of the top-level frontmatter namespace means a declared type can never
  collide with a future built-in field and `W047` needs no exemptions. Direction follows §12.1:
  the element holding the `links:` entry is the source.
- **Declared constraints**: undeclared type `E630`, malformed `links:` shape `E631`, unresolved
  target `E632`, source type not permitted `E633`, target type not permitted `E634`, too many
  targets `E635`, too few targets on an element of a declared `sourceTypes` type `W631`
  (draft-suppressed), cycle in an `acyclic` type `E636`.
- **`extends` + `relax`.** A type may `extends` one of the built-in trace links `satisfies`,
  `verifies`, `derivedFrom`, `refines`. Each instance is then treated as an instance of the base
  link by every base rule and every base reverse index, except that the codes listed in `relax`
  are not raised for that instance. `relax` may name only the base's *link-scoped* rules:

  | base | relaxable codes |
  |---|---|
  | `satisfies` | `E312`, `E313` |
  | `verifies` | `E104` |
  | `derivedFrom` | `E105`, `E310`, `W303` |
  | `refines` | `E316` |

  `coverage = false` keeps the base rule checks but withholds the instance from the base's
  reverse index (`satisfiedBy`/`verifiedBy`/`derivedChildren`/`refinedBy`), so it neither
  satisfies/verifies anything for coverage purposes nor makes its target a "parent". Built-in
  links themselves are never relaxed — relaxation is always scoped to a named variant, so
  strictness of the unadorned vocabulary is preserved and every relaxation is visible and
  reviewable in one config table.
- **Traversal is first-class.** A new `follow` command walks one named link (custom type, its
  `inverse`, or a built-in link/reverse-index name) one hop or transitively; `link-types` lists
  the declared vocabulary; `links`, `refs`, `impact`, `trace` and the MCP server all see custom
  links.
- **Suspect links.** Custom links participate in `traceBaselines:`/`W090`/`suspect` by default,
  like every other trace link (`ADR-SYS-SUSLINK-001`); `suspect = false` opts a type out.
- **LLM discoverability.** Because the vocabulary is per-project, the generic authoring prompt
  cannot enumerate it. The prompt instead tells the agent to run `link-types` (or the MCP
  `link_types` tool) and never invent a type; `--agent-instructions` given a model root appends
  the project's declared types; and `E630` names the declared types so a wrong guess
  self-corrects.

## Alternatives considered

- **`type: LinkTypeDef` model elements.** Closer to SysMLv2 metadata definitions and able to
  carry a doc body, but link vocabulary is project policy (like `[ids.prefixes]`), must be known
  before element validation, and would need its own resolution order. Rejected for now; a future
  migration could read both.
- **Top-level keys (`mitigates: [HAZ-1]`).** Terser, but collides with future built-in fields
  and needs `W047` special-casing. Rejected.
- **Relaxing built-in links model-wide.** Rejected: it silently weakens every existing link;
  a named variant keeps the relaxation explicit at each use site.
- **Web UI rendering.** Out of scope for this phase; custom links are visible through the CLI,
  MCP and the element JSON.

## Consequences

- Purely additive: no `[linkTypes]` table and no `links:` field ⇒ no new findings, no output change.
- Every relaxation is auditable in one place (`link-types` prints it).
- New codes `E630`–`E636`, `W630`, `W631` (the `E53x` range is reserved by the parked
  wasm-plugin design).
