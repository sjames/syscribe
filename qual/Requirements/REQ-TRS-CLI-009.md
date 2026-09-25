---
id: REQ-TRS-CLI-009
type: Requirement
name: The template command shall produce a valid, current-schema skeleton for every element type and list its known types from the same source
status: draft
reqDomain: software
verificationMethod: test
---

`syscribe template <Type>` (and the MCP `template` tool, which shares its implementation) **shall** print a skeleton for **every** authorable element type in the implementation's type inventory (`ElementType`) — including `Baseline` and the SysML types `OccurrenceDef`, `EventOccurrenceDef`, `ConcernDef`, `CaseDef`, `IndividualDef`, `SuccessionDef`, `RenderingDef`, `Attribute`, `Flow`, `ExhibitState`, `Concern`, `Case`, `Occurrence`, `EventOccurrence`, `Individual`, `Succession`, `BindingConnector`, `Rendering`. The only exception is `FMEAEntry`, which is never authored standalone; for it the command **shall** point the author at `template FMEASheet`.

Each skeleton **shall** use the current schema:

- the `Baseline` skeleton **shall** carry the fields `baseline create` writes (`id`, `name`, `status`, `date`, `approver`, `gitTag`, `gitCommit`, `frozenScope`, `seal`, optional `supersedes`);
- the `StateDef` skeleton **shall** use the canonical transition keys (`source`/`target`/`accept`/`guard`/`effect`, §22.1) — never the deprecated `from`/`to`/`trigger` (`W075`) — and **shall** mark an initial state (`isInitial: true`, no `W073`);
- the `TARASheet` skeleton **shall** be internally consistent: its safety-impacting damage scenario links a hazard (`hazardRef`, no `W030`) and its goal's `calLevel` meets the risk of the threats it lists (no `W032`).

When every skeleton is rendered into one scratch model (fault-tree and attack-tree children placed under their tree, as the skeletons' own comments direct), `validate` **shall** raise no findings other than unresolved references to the skeletons' placeholder names, `W005`/`W007`, and the coverage warnings whose satisfying element is deliberately outside the skeleton set (`W039`, `W613`, `W803`, `W804`, `W805`).

For an unknown type the command **shall** exit non-zero and print the list of known types, generated from the same type inventory as the skeletons, so the list can never omit a supported type (e.g. `ReviewRecord`, `TradeStudy`, `Zone`, `Conduit`, `Baseline`).

**Source:** GH #135 — `template Baseline` and eighteen SysML types reported "Unknown type"; `template StateDef` emitted deprecated keys (`W075`); `template TARASheet` tripped `W032`; the hand-written known-types list omitted `ReviewRecord`/`TradeStudy`/`Zone`/`Conduit`.

**Acceptance criteria:** `template <T>` exits 0 for every type in the inventory except `FMEAEntry`; `template Baseline` contains `gitTag:`, `frozenScope:` and `seal:`; the rendered `StateDef` raises no `W073`/`W075`; the rendered `TARASheet` raises no `W030`/`W032`; the all-templates scratch model raises only the allowed findings; `template NoSuchType` exits non-zero and its known-types list names `ReviewRecord`, `TradeStudy`, `Zone`, `Conduit` and `Baseline`.
