# Multi-Repository Composition

`GUIDE · MULTI-REPO`

Large programs partition a system model across organizational boundaries — a prime integrator assembles subcontractor models, each maintained in a separate repository. **Multi-repository composition** lets one Syscribe model import namespaces from peer repos and resolve cross-repo cross-references at analysis time, without physically merging them (§14).

Everything here is **opt-in**: a model with no `[repos]` table behaves exactly as a single repository, and none of these rules, commands, or git probes run.

The capability has three layers:

1. **Declaration** — which peer repos exist (`[repos]` in `.syscribe.toml`).
2. **Import** — which of their namespaces to mount (`repoImports:` on a package).
3. **Resolution & gating** — cross-repo references resolve, and reproducibility is enforced (`E510`–`E515`, `W510`–`W512`).

---

## 1. Declaring peer repos — `[repos]`

Peer repositories are declared in the model-root `.syscribe.toml`:

```toml
[repos]
avionics = { path = "../avionics-model", root = "model/" }
brakes   = { path = "../brakes-subsystem", root = "model/", ref = "v2.1.0" }
shared   = { path = "../shared-library", root = "model/", ref = "main" }
```

Each **table key is the repo alias** used in import declarations. Fields:

| Key | Required | Description |
|---|---|---|
| `path` | **yes** | Filesystem path to the repo root, relative to this model's `.syscribe.toml`. |
| `root` | no (default `model/`) | Where the Syscribe model lives inside the repo. |
| `ref` | no | Git tag/branch/SHA the composition is pinned to. Absent ⇒ "use whatever is on disk" (raises `W510`). |

At load time the tool resolves `<config-dir>/<path>/<root>`, walks that peer model, and indexes its element qualified names and exported stable IDs. A peer is loaded **read-only**.

---

## 2. Mounting a namespace — `repoImports:`

A package `_index.md` mounts a sub-tree from a peer repo:

```yaml
---
type: Package
name: Integration
repoImports:
  - repo: brakes        # alias from [repos]            (else E513)
    qname: BrakeSystem  # element/package in that repo  (else E514)
    as: Brakes          # local mount name (defaults to last qname segment)
---
```

This mounts the peer's `BrakeSystem` at `Integration::Brakes`.

| Field | Required | Description |
|---|---|---|
| `repo` | **yes** | Alias matching a key in `[repos]`. |
| `qname` | **yes** | Qualified name of the element/package to import, relative to the peer model root. |
| `as` | no | Local alias; defaults to the last segment of `qname`. |

---

## 3. Cross-repo resolution

Trace references — `verifies:`, `derivedFrom:`, `satisfies:`, `allocatedTo:` — resolve by searching **the local model first, then each loaded repo in declaration order**. A reference resolves by either:

- **global stable ID** (`REQ-*`, `TC-*`, …) — the stable-ID namespace is **global** across the whole composition, so `verifies: REQ-PEER-001` finds the peer's requirement with no mount prefix; or
- **qualified name** (exact, or the trailing `::`-segment of a peer qname).

```yaml
# local TestCase verifying a requirement owned by the avionics repo
id: TC-INT-001
type: TestCase
verifies:
  - REQ-AVI-NAV-014      # resolves in the avionics peer — clean, no E512
```

Cross-repo `supertype:`/`typedBy:` links are permitted for structural reuse (e.g. a shared `PartDef`). **Integrity-level propagation does not cross repo boundaries** — each repo authors its own safety case independently.

---

## 4. Reproducibility — git refs and submodules

`[repos]` is **complementary to git submodules**, not a replacement: a submodule provides the pinned checkout on disk, and `[repos]` adds the model-level resolution above plus two reproducibility gates on top. A peer can equally be a sibling checkout or a monorepo path.

- **`W511` — checkout drift.** When a repo has a `ref:`, validation compares the peer work tree's `HEAD` with the commit the `ref:` resolves to and warns when they differ ("your checkout has moved off the pinned ref").
- **`W512` — gitlink/ref disagreement.** When `path` is a git **submodule** of the composing model's repo, validation compares the commit the `ref:` resolves to against the **gitlink** the parent repo records (`git ls-tree HEAD <path>`) and warns when `.syscribe.toml` disagrees with `.gitmodules`.

Both are **opt-in advisories** by default and become a hard CI gate with `--deny`:

```bash
# fail CI unless every peer is at its pinned, consistent snapshot
syscribe -m model/ validate --deny W510 --deny W511 --deny W512
```

Neither is ever raised when the comparison can't be made — git unavailable, the peer is not a work tree, the `ref:` does not resolve, the path is absent, or (for `W512`) `path` is not a submodule — so they never false-flag a non-reproducible-by-design layout.

---

## 5. CLI

```bash
syscribe -m model/ repos [list]              # configured repos: path, ref, on-disk + sync status
syscribe -m model/ repos status              # whether each pinned repo is at its ref; exit 2 on drift
syscribe -m model/ repos sync [--all | <alias>]   # git fetch + checkout <ref> (submodule-aware checkout via the repo)
```

`repos status` and `repos list` read the same drift state the validator computes, so the CLI and `W511` never disagree.

---

## 6. Validation rules

| Code | Condition |
|---|---|
| `E510` | Circular repo import — a repo transitively imports back into this model. |
| `E511` | `repos.<alias>.path` is absent on disk **and** no `ref:` is configured. |
| `E512` | A cross-repo `verifies`/`derivedFrom`/`satisfies`/`allocatedTo` reference resolves in neither the local model nor any loaded repo. |
| `E513` | `repoImports[].repo` names an alias not present in `[repos]`. |
| `E514` | `repoImports[].qname` does not resolve to any element in the named repo. |
| `E515` | Two repos export the same stable ID (the id namespace is global). |
| `W510` | A repo has no `ref:` — composition is not pinned to a reproducible snapshot. |
| `W511` | A peer repo's `HEAD` has drifted from its configured `ref:`. |
| `W512` | A submodule peer's `ref:` disagrees with the parent's `.gitmodules` gitlink. |

See the [Rule Reference](../validation/rules.md#multi-repository-composition-e510e515-w510w512-14) for the canonical table, and §14 of the [Full Specification](../format/spec.md) for the normative definition.
