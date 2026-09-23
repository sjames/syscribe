# User-Defined Link Types

`GUIDE · LINK TYPES`

Syscribe's built-in trace links (`satisfies:`, `verifies:`, `derivedFrom:`, `refines:`, `allocatedTo:`, …) each come with a fixed set of rules. Real projects need relationships the format does not name: a safety control **mitigates** a requirement, two requirements **conflict with** each other, a software component **partially satisfies** a hardware requirement. **User-defined link types** let a project declare those relationships once, give each its own constraints, and then author, validate, traverse and suspect-check them like any built-in link (spec §12.10, `ADR-SYS-LINKTYPE-001`).

Everything here is **opt-in**. A model with no `[linkTypes]` table and no `links:` field behaves exactly as before.

The capability has three parts:

1. **Declaration.** The project's link vocabulary is declared in `[linkTypes.<name>]` tables of `.syscribe.toml`.
2. **Authoring.** Elements hold instances under a single `links:` map.
3. **Use.** Links are validated (`E630`–`E636`, `W630`, `W631`), traversed (`follow`, `links`, `refs`, `impact`, `trace`), listed (`link-types`), and suspect-checked (`W090`).

A complete worked model is in [`examples/link-types/`](https://github.com/sjames/syscribe/tree/main/examples/link-types).

---

## Why not the alternatives?

| Workaround | Problem |
|---|---|
| Abuse a built-in field (`satisfies:` for "partially satisfies") | Inherits rules that do not fit: `E313` domain matching and `E312` parent assignment fire, and the partial contributor counts as a full satisfier for coverage. |
| Put the relationship in `custom_fields:` | Nothing resolves, validates or traverses it; a typo'd id is silently accepted. |
| Invent a top-level key (`mitigates: [...]`) | Raises `W047` (unknown field), and could collide with a future built-in field. |

A declared link type avoids all three: its targets resolve like `satisfies:`, it has its own constraints, it can *selectively* inherit a built-in link's rules, and it lives in its own `links:` namespace.

---

## 1. Declaring link types: `[linkTypes]`

Declare each type as a table in the model-root `.syscribe.toml`:

```toml
[linkTypes.mitigates]
description = "A design control mitigates the failure a safety requirement guards against"
inverse     = "mitigatedBy"
sourceTypes = ["PartDef"]
targetTypes = ["Requirement"]
cardinality = "0..*"
acyclic     = false
suspect     = true
```

The table key (`mitigates`) is the **link-type name**. Names and `inverse` names are lowerCamel (`^[a-z][A-Za-z0-9]*$`) and must not collide with a built-in link or reverse-index name (`satisfies`, `verifies`, `derivedFrom`, `refines`, `supertype`, `typedBy`, `subsets`, `redefines`, `allocatedTo`, `allocatedFrom`, `satisfiedBy`, `verifiedBy`, `derivedChildren`, `refinedBy`, `specializedBy`, `links`, …), another declared type, or another type's `inverse`.

Every key is optional, and each may be written camelCase or snake_case (`sourceTypes` or `source_types`):

| Key | Type | Default | Meaning |
|---|---|---|---|
| `description` | string | none | Shown by `link-types`, the MCP `link_types` tool, and the authoring prompt. Write it for the person or agent deciding whether this link applies. |
| `inverse` | string | none | Name of the reverse direction, for `follow` and for inbound rows in `links`. Never authored. |
| `sourceTypes` | list of element types | any | Element types allowed to hold the link (`E633`). |
| `targetTypes` | list of element types | any | Element types a target may resolve to (`E634`). |
| `cardinality` | `"N"`, `"N..M"`, `"N..*"` | `"0..*"` | Number of targets per source. Upper bound: `E635`. Lower bound: `W631`. A non-zero lower bound needs `sourceTypes`. |
| `acyclic` | bool | `false` | Reject cycles of this type, including self-links (`E636`). |
| `suspect` | bool | `true` | Take part in suspect-link detection (`W090`). |
| `extends` | `satisfies` \| `verifies` \| `derivedFrom` \| `refines` | none | Make this a variant of a built-in trace link (see [section 4](#4-extending-a-built-in-link-extends-relax-coverage)). |
| `relax` | list of codes | `[]` | Base-link codes suppressed for this type only. Needs `extends`. |
| `coverage` | bool | `true` | Whether instances count in the base link's reverse index. Needs `extends`. |

**Malformed entries are ignored, loudly.** A structurally invalid entry raises `W630` naming the defect, and the entry is dropped as a whole, so any `links:` use of it becomes `E630`. Defects include a bad or colliding name, an unparseable `cardinality`, a lower bound without `sourceTypes`, an unknown element type, an unsupported `extends`, `relax`/`coverage` without `extends`, and a non-relaxable `relax` code. An unknown key (a typo such as `colour`) is also `W630`; only that key is ignored. One bad entry never disables the rest of the table.

---

## 2. Authoring links: `links:`

Any element may hold instances under `links:`, a map from a declared type name to one reference or a list of references. References are stable ids or qualified names, resolved exactly as `satisfies:` targets are:

```yaml
---
type: PartDef
name: WatchdogMonitor
domain: software
links:
  mitigates: [REQ-BRK-002, REQ-BRK-003]   # list form
---
```

```yaml
---
type: Requirement
id: REQ-BRK-004
name: "Regenerative braking share maximised during service braking"
status: approved
reqDomain: software
links:
  conflictsWith: REQ-BRK-002              # single-reference form
---
```

**Direction follows the OSLC rule (§12.1).** The element holding the `links:` entry is the **source**, and the reference is the **target**. You never write the reverse direction; the declared `inverse` is computed. If you find yourself wanting to write a link on the "upstream" end, declare the type the other way round.

Shape errors are caught before anything else: an undeclared key is `E630`, a non-map `links:` or a non-string value is `E631`, and a dangling reference is `E632`.

---

## 3. Declared constraints

| Declaration | Check |
|---|---|
| `sourceTypes = ["PartDef"]` | A `links: { mitigates: … }` on anything other than a `PartDef` is `E633`. |
| `targetTypes = ["Requirement"]` | A `mitigates` target that resolves to anything other than a `Requirement` is `E634` (the target is named). |
| `cardinality = "0..2"` | A third target on one source is `E635`. |
| `cardinality = "1..*"` + `sourceTypes` | A non-draft element of a listed source type with no target is `W631`. Draft elements are skipped. Gate with `--deny W631`. |
| `acyclic = true` | `A → B → A`, or `A → A`, through that type alone is `E636`, reported once per cycle. |

A lower bound turns a link type into a completeness policy. For example, "every `SecurityControl` must mitigate at least one threat" is:

```toml
[linkTypes.mitigatesThreat]
description = "The security control mitigates this threat scenario"
inverse     = "mitigatedByControl"
sourceTypes = ["SecurityControl"]
targetTypes = ["ThreatScenario"]
cardinality = "1..*"
```

---

## 4. Extending a built-in link: `extends`, `relax`, `coverage`

Sometimes a relationship *is* a built-in link, just a weaker or specialised one. `extends` makes a type a **named variant** of `satisfies`, `verifies`, `derivedFrom` or `refines`. Each instance is then treated as an instance of the base link by every base rule and every base reverse index (`satisfiedBy`, `verifiedBy`, `derivedChildren`, `refinedBy`), and so by the coverage checks built on them (`W002`, `W300`, `W305`, …), by `impact` and `trace`, and by suspect detection. Its own declared constraints apply as well.

`relax` then switches off specific base rules **for that variant only**. Only a base's link-scoped rules can be relaxed:

| `extends` | Relaxable codes |
|---|---|
| `satisfies` | `E312` (parent assignment), `E313` (domain mismatch) |
| `verifies` | `E104` (target kind) |
| `derivedFrom` | `E105` (target kind), `E310` (no `breakdownAdr:`), `W303` (proposed ADR) |
| `refines` | `E316` (target kind) |

`E310` is judged per requirement: it is suppressed only when **every** `derivedFrom`-like link on that requirement, including the built-in field, is of a type that relaxes `E310`.

`coverage = false` keeps the base rule checks but withholds instances from the base reverse index. The link is recorded and traversable, but it does not satisfy or verify anything for coverage purposes (`W002`, `W300`, `W305`, `W015`, `W614` and the coverage reports never credit it), and it does not make its target a "parent". Rules that only check that a trace link *exists* still count it: a TestCase whose only link is a `coverage = false` `verifies` variant does not raise `E013`, and a requirement linked upstream only that way is not an orphan (`W005`).

An extending link whose target lives in a `[repos]` peer is resolved and traversable, but the base's rules are only applied to targets in the local model.

**Built-in fields are never relaxed.** A declaration only affects its own named variant. A plain `satisfies:` next to a relaxing variant still raises `E313`. Every relaxation in a project is visible in one place, `.syscribe.toml`, and printed by `link-types`.

### Worked example: `partiallySatisfies`

A brake controller (software) contributes to a hydraulic pressure-release requirement (hardware), but the hydraulic modulator is what actually satisfies it. With only built-in links you can either:

- write `satisfies: [REQ-BRK-003]` on the controller, which raises `E313` (software element, hardware requirement); it would also count the controller as a full satisfier, which overstates its role; or
- leave the contribution out of the model entirely.

Declare a variant instead:

```toml
[linkTypes.partiallySatisfies]
description = "A cross-domain contributor; the owning-domain element holds the real satisfies:"
inverse     = "partiallySatisfiedBy"
extends     = "satisfies"
relax       = ["E313"]
coverage    = false
```

```yaml
# Architecture/HydraulicModulator.md
type: PartDef
name: HydraulicModulator
domain: hardware
satisfies: [REQ-BRK-003]                 # the real, same-domain assignment
```

```yaml
# Architecture/BrakeController.md
type: PartDef
name: BrakeController
domain: software
satisfies: [REQ-BRK-002]                 # software → software
links:
  partiallySatisfies: REQ-BRK-003        # software → hardware: E313 relaxed
```

What each setting does here:

- **`extends = "satisfies"`**: every other `satisfies` rule still applies. If `REQ-BRK-003` were broken down into children, this link would raise `E312` like any `satisfies`.
- **`relax = ["E313"]`**: the cross-domain instance is accepted. The controller's own `satisfies: [REQ-BRK-002]` is still domain-checked.
- **`coverage = false`**: `REQ-BRK-003`'s `satisfiedBy` is still just `HydraulicModulator`, so coverage reports don't credit the controller. If the modulator's `satisfies:` were removed, `W300` would fire, because a partial contribution is not coverage.

To trace the partial contributors, use the inverse:

```bash
syscribe -m model/ follow REQ-BRK-003 partiallySatisfiedBy
syscribe -m model/ follow REQ-BRK-003 satisfiedBy          # only the full satisfier
```

Set `coverage = true` (the default) when the variant *should* count. For example, a `satisfiesByAnalysis` variant that relaxes nothing, but that you want to keep distinguishable from plain `satisfies` in reports and in `follow`.

---

## 5. Following links

### `follow`

```
syscribe -m <root> follow <elem> <link> [--reverse] [--transitive] [--depth N] [--format text|json|dot]
```

`follow` walks one named link from `<elem>` (an id or qualified name). `<link>` may be:

- a declared link type (walks forward, source to target);
- a declared `inverse` (walks backward);
- a built-in link: `satisfies`, `verifies`, `derivedFrom`, `refines`, `supertype`, `typedBy`, `allocatedTo`;
- a built-in reverse index: `satisfiedBy`, `verifiedBy`, `derivedChildren`, `refinedBy`, `specializedBy`, `allocatedFrom`.

`--reverse` flips the direction, so `follow REQ-BRK-003 mitigates --reverse` is the same walk as `follow REQ-BRK-003 mitigatedBy`. One hop is followed by default. `--transitive` follows to a fixed point and stops on cycles. `--depth N` bounds the hops and implies `--transitive`. Each reached element is listed once, with its hop depth, id/qname, type and name.

```bash
syscribe -m model/ follow Architecture::WatchdogMonitor mitigates
syscribe -m model/ follow REQ-BRK-003 mitigatedBy
syscribe -m model/ follow REQ-BRK-004 conflictsWith --transitive
syscribe -m model/ follow REQ-BRK-001 derivedChildren --depth 3 --format dot | dot -Tsvg > tree.svg
```

`--format json` emits an object of the form `{start, link, direction, results: [{qname, id, type, name, depth, from}]}`. An illustrative result:

```json
{
  "start": "REQ-BRK-003",
  "link": "mitigatedBy",
  "direction": "reverse",
  "results": [
    { "qname": "Architecture::PressureReliefValve", "id": null, "type": "PartDef",
      "name": "PressureReliefValve", "depth": 1, "from": "REQ-BRK-003" },
    { "qname": "Architecture::WatchdogMonitor", "id": null, "type": "PartDef",
      "name": "WatchdogMonitor", "depth": 1, "from": "REQ-BRK-003" }
  ]
}
```

An unknown element or link name exits non-zero, and an unknown link name prints the names that are available.

### `link-types`

```
syscribe -m <root> link-types [--json]
```

This lists every valid declared type with its description, inverse, `extends` base, relaxed codes, coverage, source and target types, cardinality, `acyclic` and `suspect` settings, and how many instances the model holds. An entry rejected under `W630` is not listed as usable. With nothing declared, it says so, shows how to declare a type, and exits zero.

### Existing commands

| Command | Custom-link behaviour |
|---|---|
| `links <elem>` | Outbound instances appear under the type name. Inbound instances appear under the `inverse`, or as `<type> (inbound)` when no inverse is declared. |
| `refs <elem>` | Includes inbound custom links. |
| `impact <elem>` | Traverses custom links: upstream along the link, downstream against it. `--kinds` accepts custom type names, for example `--kinds mitigates,partiallySatisfies`. |
| `trace <req>` | Lists the custom links touching the requirement. |
| `show <elem>` | Displays the element's `links:`. |

### MCP

The MCP server exposes two read-only tools:

- **`link_types`** returns the same data as `link-types --json`.
- **`follow`** takes `element` and `link`, plus optional `reverse`, `transitive` and `depth`, and returns the same data as `follow --format json`.

The generic `create_element`/`update_element` tools accept a `links:` field. Mutating commands never rewrite a variant link into its base field.

---

## 6. Suspect links

Custom links are trace links for [suspect-link detection](../design/suspect-links.md). `suspect list` shows them, `suspect accept` baselines them into the source's `traceBaselines:`, and `validate` raises `W090` when a baselined target's normative content changes:

```bash
syscribe -m model/ suspect accept --all-unbaselined   # baseline every link, custom ones included
# ... edit REQ-BRK-003's normative text ...
syscribe -m model/ validate --deny W090               # mitigates / partiallySatisfies links now flagged
```

Set `suspect = false` on types whose validity does not depend on the target's exact wording, such as `conflictsWith` or `informs`. They are then left out of `suspect list`, `suspect accept` and `W090`.

---

## 7. LLM-assisted authoring

The link vocabulary differs from project to project, so the generic authoring prompt cannot list it. An agent should discover it from the tool before it writes any link:

1. Run `syscribe -m <root> link-types` (or call the MCP `link_types` tool), **or**
2. Read the **"Project link types"** section that `syscribe -m <root> --agent-instructions` appends when the model declares any types.

The prompt tells the agent never to invent an undeclared type. If it does, `E630` lists the declared types, so the next validation pass corrects it. The MCP server's `initialize` instructions also point clients at `link_types`.

Recommended practice: give every type a `description` that says *when* to use it. That text is exactly what an agent reads when choosing between, say, `satisfies:` and `partiallySatisfies`.

---

## 8. Validation codes

| Code | Severity | Condition |
|---|---|---|
| `E630` | error | `links:` key is not a declared (valid) link type; the message lists the declared types. |
| `E631` | error | `links:` is not a map, or a value is not a string or a list of strings. |
| `E632` | error | A `links:` reference does not resolve. |
| `E633` | error | Holding element's type is not in the link type's `sourceTypes`. |
| `E634` | error | A target's type is not in the link type's `targetTypes`. |
| `E635` | error | More targets than the `cardinality` upper bound. |
| `E636` | error | Cycle (or self-link) in an `acyclic = true` type. |
| `W630` | warning | Malformed `[linkTypes]` entry (ignored as a whole), or unknown key in an entry (key ignored). |
| `W631` | warning | Non-draft element of a `sourceTypes` type is below the `cardinality` lower bound. |

Variant types additionally raise their base link's codes (`E104`, `E105`, `E310`, `E312`, `E313`, `E316`, `W303`, …) except those in `relax`, and custom links raise `W090` when suspect. See the [Rule Reference](../validation/rules.md#user-defined-link-types-e630e636-w630-w631-1210-adr-sys-linktype-001) and spec §12.10.
