---
type: Requirement
id: REQ-TRS-EXPRES-001
name: "export --resolved carries what a document generator needs from one call"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - export
---

`export --resolved` shall add to the export what a document generator would otherwise re-derive from the model files (GH #529), so a report can be built from one call and no file access to the model. Without `--resolved` the export shall be exactly what it was.

1. {#body} Every element carries `body`: `raw`, its Markdown body, and `rendered`, the body with each value reference (`REQ-TRS-ATTRREF-001`) replaced by its value and unit. A reference that does not resolve is left as written.
2. {#params} Every `Configuration` carries `effectiveParameters`: one row per parameter that has a value in it, with `key`, `feature`, `featureId`, `name`, `value`, the declared `unit`, `type`, `range` (`{min, max}`), `bindingTime` and `latestBindingTime`, the `source` of the value, and for a derived parameter the `expression` beside its computed value. The value and the unit are separate members.
3. {#config} With `--config <C>` the export is the projection onto `C`: a body holds `C`'s value for each feature-parameter placeholder, the top level carries `C`'s `effectiveParameters`, and `meta.config` is its id.
4. {#reverse} `computed` gains `satisfiedBy` (the architecture elements satisfying a requirement) and `citedBy` (`[{by, ref}]`: the elements whose body cites the element, one of its items or one of its questions, with the reference as written), beside `verifiedBy` and the other reverse links.
5. {#questions} A `PlanningItem` with questions carries `questions` (each with its `ref`, `PI-X#Qn`, its status, text, recommendation and answer) and `openQuestions`, the number that are open.
6. {#meta} The top level carries `resolved: true` and `meta`: `commit` and `dirty` of the repository holding the model (`null` outside git), `syscribeVersion`, `generatedAt` (UTC, RFC 3339) and `config`.
7. {#ndjson} With `--ndjson` the header line carries the top-level members and each element line its own.
8. {#schema} `syscribe spec export` documents every member. The additions do not change `schemaVersion`; it changes when a member changes meaning or is removed.
