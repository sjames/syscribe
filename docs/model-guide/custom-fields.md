# Custom Fields

Sometimes you need to attach data to a model element that the Syscribe schema does not
define — a supplier name, a cost centre, a maturity tag, a list of part numbers. The
`custom_fields:` map is the intentional, addressable home for that data.

Without it, any unrecognised top-level frontmatter key is silently swallowed: no
namespace, no validation, no way to query it, and no way to tell a typo'd schema field
from deliberate custom data. `custom_fields:` fixes that.

## Declaring custom fields

`custom_fields:` is a flat map of `string -> scalar | list-of-scalars`, accepted on
**every** element type:

```yaml
---
id: VehicleSystem::Powertrain::Engine
type: PartDef
name: Engine
custom_fields:
  supplier: Bosch
  costCenter: PWT-4471
  maturity: prototype
  reviewCycle: 3
  partNumbers: [A-1001, A-1002]   # lists of scalars allowed
---
```

- **Keys are freeform.** No registration, no name validation — this fits the
  LLM-authoring workflow.
- **Values must be scalars or lists of scalars.** A nested map, or a list containing a
  map, raises warning `W041` (`custom field '<key>' must be a scalar or a list of
  scalars`). `W041` is advisory; gate it in CI with `--deny W041`.
- The map serialises in **sorted key order**, so editing one element never produces a
  noisy round-trip diff.
- Elements with no `custom_fields:` are completely unaffected.

## Querying custom fields — `--where`

The `ls`, `find`, and `list` commands accept a `--where` predicate that addresses a
custom field via the `custom.<key>` namespace:

```bash
syscribe -m model/ ls PartDef --where custom.supplier=Bosch     # exact match
syscribe -m model/ find --where custom.maturity=prototype       # exact match
syscribe -m model/ ls --where custom.costCenter=~PWT            # regex / substring
syscribe -m model/ ls --where custom.partNumbers~=A-1001        # list membership
syscribe -m model/ ls --where custom.supplier                   # presence (field is set)
```

| Operator | Meaning |
|---|---|
| `custom.<key>=<value>` | exact (scalar equals, or any list element equals) |
| `custom.<key>=~<pattern>` | regex match on the value's string form (falls back to substring if the pattern is not a valid regex) |
| `custom.<key>~=<value>` | list membership (the list contains the value; or the scalar equals it) |
| `custom.<key>` | presence — the field is set, any value |

`--where` composes (logical AND) with the existing `type` / `--tag` / `--status`
filters, and you may supply more than one `--where` (also ANDed). An unparseable
predicate is a usage error and exits non-zero.

## Viewing custom fields

Custom fields are **read-only**. They are rendered, but never editable:

- `syscribe -m model/ show <qname>` prints a **Custom Fields** section (scalars inline,
  lists comma-joined) when the element declares any.
- The web UI element **detail panel** shows a read-only key/value table. The
  `PUT /api/elements/<qname>` editor does not expose them.
