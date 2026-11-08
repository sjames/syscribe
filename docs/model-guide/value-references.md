# Value references

A design number is often repeated across elements: a size budget in a design element, the
same figure in a requirement, again in a TestCase scenario and in a planning item. When it
changes, every copy has to be found by hand. A **value reference** states the value once and
cites it everywhere else (GH #452, spec §5.8):

```markdown
The policy snapshot is {{PolicydBounds.snapshotFixedBytes}} long.
```

`show`, `export-html` and the web detail panel print the attribute's current value with its
unit — `The policy snapshot is 47244 B long.` — while the file keeps the reference. When the
budget changes, every rendering follows.

## Syntax

`{{Elem.attr}}`

- `Elem` names the element, resolved in this order: a stable id (`REQ-…`, `TC-…`), an exact
  qualified name (`Arch::PolicydBounds`), then its **basic name** — the last qualified-name
  segment (`PolicydBounds`) — when exactly one element in the model has it. If several elements
  share the basic name, the reference is `E331` and the message lists their qualified names;
  write the qualified name or the id instead. No display-name or fuzzy matching.
- `attr` names one attribute of it, as a dotted path where values nest
  (`{{PolicydBounds.snapshot.header}}`, `{{PolicydBounds.Budget.limitKb}}`).
- An optional `|unit` suffix is accepted; the unit is always rendered.

## What counts as an attribute

Every named slot of an element that holds a single scalar value:

| Form | Reference | Value rendered |
|---|---|---|
| `features:` entry with `value:` (else `default:`) | `{{Elem.name}}` | the value and its `unit:` |
| nested `features:` entry | `{{Elem.outer.inner}}` | the value and its `unit:` |
| `Attribute` child element (`Elem/maxClients.md`) | `{{Elem.maxClients}}` | its `value:` (an `AttributeDef`: `default:`) and `unit:` |
| stereotype tagged value (`metadata:`) | `{{Elem.Budget.limitKb}}`, or `{{Elem.limitKb}}` when one application has the key | the value, else the `MetadataDef` default, and the feature's `unit:` |
| budget / timing field | `{{Elem.ram}}`, `flash`, `nvm`, `stack`, `*Capacity`, `margin`, `period`, `deadline`, `priority`, `cpuMargin`, `wcet`, `failureRate`, `diagnosticCoverage` | the field as written |

```yaml
---
type: PartDef
name: PolicydBounds
features:
  - name: snapshotFixedBytes
    typedBy: ScalarValues::Integer
    value: 47244
    unit: B
---
```

A list, a map, or a feature with neither `value:` nor `default:` (a port) is an attribute
without a value; referencing it is `E331`.

## Where references are read

In the Markdown body, outside code. Inline code spans and fenced blocks keep the text literal,
so documentation can show the syntax — **except** fenced `gherkin` (also `feature`,
`cucumber`) blocks, which hold TestCase scenarios and are read:

````markdown
```gherkin
Scenario: the snapshot fits
  Given a snapshot of {{PolicydBounds.snapshotFixedBytes}}
  Then it is accepted
```
````

Inline code spans inside a scenario block stay literal. The frontmatter and the element's
`name` are not read.

`{{Features::Display.sizeInch}}`-style references to a `FeatureDef` parameter are
[feature-parameter placeholders](variability.md) and keep their own rules: a reference whose
element is a `FeatureDef`, or a `::` path / `FEAT-` id that resolves to nothing, is a placeholder
(`E240`/`E241`), not a value reference. An unresolved bare name starting with a lowercase letter
(`{{ user.name }}`) is template text and is ignored.

## Checks

| Code | When |
|---|---|
| `E331` | A value reference does not resolve: the element is unknown, the attribute is unknown (the message lists the attributes that have a value), or the attribute has no single scalar value |
| `W095` | Opt-in. A numeric literal in the body equals the **current** value of an attribute of an element the holder links to |
| `W096` | A non-draft element's body holds a numeric literal equal to a **previous** value of such an attribute — a stale copy, found in git history |

*Links* are every outgoing reference: `satisfies`, `verifies`, `verifiesElement`,
`derivedFrom`, `refines`, `allocatedTo`/`allocatedFrom`, `typedBy`, `supertype`, `achieves`,
`evidence` `ref:`, user-defined `links:`, and the elements of the body's own value references.

### Avoiding noise

A literal is compared only when it is a whole token — not part of an id (`TC-47244`), a version
(`v12.5.1`), a date (`2026-11-07`), a time (`12:30`) or a word; a unit directly after it
(`10ms`) is fine. Thousands separators are read, so `47 244`, `47,244`, `47_244` and `47'244`
all match `47244`. Values below **10** and integers from **1900 to 2100** (years) are never
compared. An attribute value is compared when it is a plain number, optionally followed by a
unit word (`12 KiB` compares as 12); magnitudes are compared as written, without unit
conversion. Code is skipped exactly as for references.

### Turning on copied-value checks

```toml
# .syscribe.toml
[references]
copied_values = true
```

`copied_values` must be `true` or `false`; anything else is reported as `W094` against
`.syscribe.toml` and ignored. `lint-docs` applies `W095` (advisory) to external Markdown, where
a document's links are the elements it cites with value references.

### Stale copies

`W096` needs no configuration. For each attribute an element links to, the tool reads the
committed versions of the file that holds it — one `git log --follow` and one
`git cat-file --batch` per file, at most `[agents] audit_limit` commits (default 200), cached
per file — and collects the values it held before. A literal equal to one of them, and to no
current value, is a stale copy: `46 852` left in a TestCase after the budget became `47 244`.
Draft elements are exempt; without git nothing is reported.

## Finding every use: `refs Elem.attr`

```
syscribe -m <root> refs PolicydBounds.snapshotFixedBytes
syscribe -m <root> refs PolicydBounds.snapshotFixedBytes --json
```

prints the value and three sections — *Uses* (every `{{…}}`), *Literal copies of the current
value* and *Stale copies of previous values* (with the previous value and the commit that held
it) — over the elements that link to `PolicydBounds`. Replace each copy with the reference.
