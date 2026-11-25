# `export` — schema of the model graph

`syscribe -m <root> export [--resolved] [--ndjson] [--config <C>]` writes the model as JSON.
`schemaVersion` is `"1.0"`. It changes when a member changes meaning or is removed; a new member
does not change it, so a reader must ignore members it does not know.

## Document

| Member | Type | Contents |
|---|---|---|
| `schemaVersion` | string | Version of this schema |
| `elementCount` | number | Number of entries in `elements` |
| `elements` | array | One object per element (below) |
| `resolved` | `true` | Only with `--resolved` |
| `meta` | object | Only with `--resolved` (below) |
| `effectiveParameters` | array | Only with `--resolved --config <C>`: the parameter rows of `<C>` (below) |

With `--ndjson` the first line is a header (`kind: "header"`, and every document member except
`elements`), followed by one element object per line.

## Element

| Member | Type | Contents |
|---|---|---|
| `qname` | string | Qualified name |
| `file` | string | Source file |
| `url` | string | Hosted source URL, when `[links]` is configured |
| `id` | string | Stable id, when the element has one |
| `type` | string | Element type |
| `name` | string | Label |
| `source`, `target`, `ends` | | A connection's endpoints |
| `frontmatter` | object | The frontmatter fields that are set |
| `computed` | object | Reverse links, each a list, present when not empty: `verifiedBy`, `derivedChildren`, `refinedBy`, `actorIn`, `mopRefinedBy`, `allocatedFrom` |

`--resolved` adds what a document generator would otherwise re-derive from the files:

| Member | On | Contents |
|---|---|---|
| `body.raw` | every element | The Markdown body as written. Under `--config` it is the projected text: a feature-parameter placeholder `{{Features::F.p}}` already holds that Configuration's value |
| `body.rendered` | every element | `body.raw` with each value reference `{{Elem.attr}}` replaced by its value and unit. A reference that does not resolve is left as written (`validate` reports it as `E331`) |
| `computed.satisfiedBy` | Requirement | The architecture elements whose `satisfies:` names it (id, else qualified name) |
| `computed.citedBy` | any | The elements whose body cites it, an item of it (`REQ-X#slug`) or a question of it (`PI-X#Q1`): `[{by, ref}]`, `ref` as written |
| `questions` | PlanningItem | Its questions: `ref` (`PI-X#Q1`), `id`, `text`, `status`, `recommended`, `answer`, `answeredBy`, `answeredAt`, `answerBy` (`human` or `any`). Absent when it has none |
| `openQuestions` | PlanningItem | How many of them have `status: open` |
| `effectiveParameters` | Configuration | Its parameter rows (below) |

## Parameter row

One row per parameter that has a value in the Configuration: bound by it or an ancestor, fixed,
derived, propagated by `bindTo:` or defaulted. The value and the unit are separate members, so a
generator formats the quantity for its target.

| Member | Contents |
|---|---|
| `key` | `<FeatureDef qualified name>.<parameter>` |
| `feature`, `featureId`, `name` | The feature (qualified name and `FEAT-*` id) and the parameter name |
| `value` | The effective value. For a derived parameter the computed value, or the expression text while it cannot be computed |
| `unit`, `type`, `description` | From the parameter declaration, when declared |
| `range` | `{min, max}` from the declared `range:` |
| `bindingTime`, `latestBindingTime` | From the declaration, when declared |
| `source` | `own`, `inherited`, `subConfiguration`, `bindTo`, `derived`, `fixed` or `default` |
| `from` | The Configuration (`inherited`) or system parameter (`bindTo`) the value came from |
| `expression` | The `derivedFrom:` expression of a derived parameter |
| `stage` | The binding stage of the Configuration that bound it, when it has one |

## `meta`

| Member | Contents |
|---|---|
| `commit` | The `HEAD` commit of the repository holding the model; `null` outside git |
| `dirty` | Whether that work tree has uncommitted changes; `null` outside git |
| `syscribeVersion` | The version of syscribe that wrote the export |
| `generatedAt` | UTC time of the export, RFC 3339 |
| `config` | The id of the `--config` Configuration, else `null` |
