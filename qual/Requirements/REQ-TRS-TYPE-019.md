---
id: REQ-TRS-TYPE-019
type: Requirement
name: Tool shall recognise, validate, and score the native TradeStudy element (TRD-*)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise a native **`TradeStudy`** element (§15, GH #63) — a
general-purpose weighted-criteria evaluation of design alternatives, independent of the
MagicGrid profile. It is id-identified (`TRD-*`).

**Frontmatter:** `id` (TRD-*), `name`, `status` (`draft|review|complete`), required
`criteria:` (each `name`/`weight`/`direction`), `alternatives:` (each `name`, optional
`element`), `scores:` (each `alternative`/`criterion`/`score`), and optional `objective:`
(Requirement) / `decision:` (ADR).

### Computed scoring (§15.3)

The tool **shall** compute, but never write: min-max **normalised** score per criterion
column (direction applied — best = 1, worst = 0), **weighted** by the normalised criterion
weight, a **total** per alternative, and a **rank** (total descending).

### Validation rules

| Code | Condition |
|---|---|
| `E869` | Missing `id`, `name`, `status`, `criteria`, `alternatives`, or `scores`. |
| `E870` | `id` does not match the `TRD-*` pattern. |
| `E871` | A `criteria:` entry is missing `name`, `weight`, or `direction`. |
| `E872` | A `criteria[].weight` is not in [0.0, 1.0], or all weights are zero. |
| `E873` | A `criteria[].direction` is not `maximize` or `minimize`. |
| `E874` | `alternatives:` is empty. |
| `E875` | An `alternatives:` entry is missing `name`. |
| `E876` | A `scores:` entry references an unknown alternative or criterion. |
| `E877` | A `scores[].score` is not a number. |
| `W061` | A `status: complete` study has no `decision:` ADR. |
| `W062` | `objective:` is present but unresolved (draft-suppressed). |
| `W063` | The score matrix is incomplete — some alternative×criterion pair has no score (draft-suppressed). |
| `W064` | An `alternatives[].element` is present but unresolved (draft-suppressed). |

### CLI

`trade-study [<TRD-id>] [--json]` **shall** list all `TradeStudy` elements, or print one
study's full normalised scoring table and ranking. When the model contains `TradeStudy`
elements this command **shall** take precedence; the MagicGrid MoE `trade-study` (REQ-TRS-MG-007)
remains for models without them. `template TradeStudy` **shall** be available.

> **Code note:** the spec drafted these as `E400`–`E408` / `W400`–`W403`, which collide with
> the Diagram codes (`E400`–`E402`, `W400`–`W403`). They are reassigned to **`E869`–`E877`**
> and **`W061`–`W064`**; the spec §15.5 is corrected.

**Source:** §15 (General-Purpose Trade Study), GH #63.

**Acceptance criteria:**

- A well-formed `TradeStudy` validates clean; `E869`–`E877` fire on the matching defects.
- `W061` fires for a complete study with no `decision`; `W063` for an incomplete matrix.
- `trade-study <TRD-id>` prints a ranked, normalised scoring table; `--json` is structured.
- `trade-study` lists studies; `template TradeStudy` produces a valid skeleton.
- Shipped models (no `TradeStudy`) are unaffected; the MagicGrid `trade-study` still works.
