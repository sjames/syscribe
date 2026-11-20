---
type: Requirement
id: REQ-TRS-SETNAME-001
name: "set changes the name of an id-identified element, and evidence.add path= resolves repo: paths as validate does"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - planning
---

`syscribe set <id> name=<text>` shall change the `name:` of an id-identified element in place, through the same candidate-copy validation as every other `set` field, and shall refuse an empty or multi-line name. For a name-identified element (`PartDef`, `Package`, `FeatureDef`, …), where `name` is the last segment of the qualified name, it shall refuse and name `move` as the command that renames and rewrites every reference. It shall list the other elements whose text quotes the old name, since prose is not rewritten (GH #502).

`syscribe set <id> evidence.add path=<path>` shall resolve `<path>` exactly as `validate` resolves an evidence `path:` (`classify_source`, the logic of `implementedBy:` and `W023`): `repo:<path>` relative to the repository, `model:<path>` and a bare path relative to the model root, an absolute path as is, a URI or package reference external. A refusal shall name the location it looked at.
