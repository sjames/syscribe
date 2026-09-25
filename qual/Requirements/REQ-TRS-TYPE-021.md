---
id: REQ-TRS-TYPE-021
type: Requirement
name: Tool shall support multi-repository model composition
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** support **multi-repository model composition** (§14, GH #62): a model
**shall** declare peer repositories in a `[repos]` table of the model-root `.syscribe.toml`
and import namespaces from them, resolving cross-repo cross-references at analysis time.

- **`[repos]` table** — each entry key is a repo **alias**; fields are `path` (**required**,
  relative to this model's `.syscribe.toml`), `root` (optional, default `model/`) and `ref`
  (optional git tag/branch/SHA).
- **`repoImports:`** on a Package `_index.md` mounts a peer sub-tree: a list of
  `{repo, qname, as}` mappings, where `repo` is an alias from `[repos]`, `qname` is the
  element/package to import, and `as` is the optional local mount name (default: the last
  segment of `qname`). The peer subtree **shall** be mounted at `<package-qname>::<as>` (just
  `<as>` on the model-root package): a reference `<package-qname>::<as>::X` **shall** resolve
  to the peer's `<qname>::X` wherever cross-repo resolution applies (the most specific mount
  wins when mounts nest).
- **Cross-repo resolution (§14.4)** — `verifies:`/`derivedFrom:`/`satisfies:`/`allocatedTo:`
  references and the structural `supertype:`/`typedBy:`/`subsets:`/`redefines:` references
  resolve by searching the local model first, then each loaded repo in declaration order, by
  **global stable ID**, by the peer-native qualified name, or through a mount point. The
  stable-ID namespace is global across the composition.

### Validation rules

| Code | Condition |
|---|---|
| `E510` | Circular repo import — a repo transitively imports back into this model. |
| `E511` | `repos.<alias>.path` is absent on disk and no `ref:` is configured. |
| `E512` | A cross-repo `verifies`/`derivedFrom`/`satisfies`/`allocatedTo`/`supertype`/`typedBy`/`subsets`/`redefines` reference (including one written through a mount point) resolves in neither the local model nor any loaded repo. |
| `E513` | `repoImports[].repo` names an alias not present in `[repos]`. |
| `E514` | `repoImports[].qname` does not resolve to any element in the named repo. |
| `E515` | Two repos export the same stable ID — the local model and a peer, or two different peer repos (the id namespace is global). |
| `W510` | A repo in `[repos]` has no `ref:` — composition is not pinned (opt-in; `--deny W510`). |

### CLI

`repos list [--json]` (configured repos with path/ref/on-disk + sync status),
`repos status [--json]` (whether each pinned repo is at its `ref`; exits 2 on drift), and
`repos sync [--all | <alias>]` (`git fetch` + `git checkout <ref>` for pinned repos).

**Source:** §14 (Multi-Repository Model Composition), GH #62.

**Acceptance criteria:**

- `[repos]` and `repoImports:` parse; the cross-repo block is inert for single-repo models.
- `E510`–`E515` fire on the matching defects; a valid composition with a cross-repo
  `verifies:` is clean (no `E102`/`E512`).
- A composition referencing peer elements through `<package>::<as>::X` for `verifies`,
  `derivedFrom`, `satisfies`, `allocatedTo`, `supertype` and an inline-feature `typedBy` is
  clean, as are peer-native qname and stable-id references; a mounted reference naming nothing
  in the peer is `E512`; a stable ID exported by two different peers is `E515`.
- `W510` fires for a repo with no `ref:`.
- `repos list` reflects the configured repos.
