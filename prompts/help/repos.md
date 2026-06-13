# repos — multi-repository composition (§14)

Inspect and synchronise the peer repositories declared in the model-root
`.syscribe.toml` `[repos]` table. Large programs partition a system model across
organizational boundaries; multi-repo composition lets one model import namespaces
from peer repos and resolve cross-repo cross-references at analysis time.

## Configuration (`.syscribe.toml`)

```toml
[repos]
avionics = { path = "../avionics-model", root = "model/" }
brakes   = { path = "../brakes-subsystem", root = "model/", ref = "v2.1.0" }
shared   = { path = "../shared-library", root = "model/", ref = "main" }
```

- `path` (**required**) — file-system path to the repo root, relative to this
  model's `.syscribe.toml`.
- `root` (optional, default `model/`) — where the Syscribe model root lives inside
  the repo.
- `ref` (optional) — git tag/branch/SHA to check out via `repos sync`; absent means
  "use whatever is on disk" (and raises `W510`, since the snapshot is not pinned).

## Import declarations (`_index.md`)

```yaml
type: Package
name: Integration
repoImports:
  - repo: avionics       # alias from [repos]
    qname: Avionics      # package/element to import from that repo
    as: Avionics         # local mount name (defaults to the last qname segment)
```

## Subcommands

```
repos                # alias for `repos list`
repos list [--json]  # configured repos: path, ref, on-disk + sync status
repos status [--json] # whether each pinned repo is at its ref; exit 2 if any drifted
repos sync [--all | <alias>]  # git fetch + checkout <ref> for pinned repos
```

## Resolution & validation

Cross-repo `verifies:` / `derivedFrom:` / `satisfies:` / `allocatedTo:` references
resolve by searching the local model first, then each loaded repo in declaration
order — by global stable ID (`REQ-*`, `TC-*`, …) or by qualified name. Stable IDs
are **global** across the composition.

| Code | Condition |
|---|---|
| `E510` | Circular repo import — a repo transitively imports back into this model |
| `E511` | `repos.<alias>.path` is absent on disk and no `ref:` is configured |
| `E512` | Cross-repo trace reference resolves in neither the local model nor any repo |
| `E513` | `repoImports[].repo` names an alias not present in `[repos]` |
| `E514` | `repoImports[].qname` does not resolve in the named repo |
| `E515` | Two repos export the same stable ID (the id namespace is global) |
| `W510` | A repo has no `ref:` — composition is not pinned (gate with `--deny W510`) |
| `W511` | A pinned repo's `HEAD` has drifted from its `ref:` (gate with `--deny W511`) |
| `W512` | A submodule peer's `ref:` disagrees with the parent's `.gitmodules` gitlink (gate with `--deny W512`) |

## Reproducibility gate

When a repo declares a `ref:`, validation compares the peer work tree's `HEAD` with the
commit the `ref:` resolves to and emits `W511` on drift. Gate CI on a pinned composition
with `validate --deny W511`. `repos status` reports the same drift and exits `2`; run
`repos sync <alias>` (or `--all`) to bring the checkout back to its `ref:`. Drift is never
reported when it cannot be determined (git unavailable, not a work tree, `ref:` unresolved).

## Git submodules

`[repos]` and git submodules are complementary: a submodule provides the pinned checkout on
disk, and `[repos]` adds model-level cross-reference resolution plus the reproducibility
gates on top — so `W511`/`W512` work out of the box against a submodule. When a repo's
`path` is a submodule of the composing model's repository, `W512` compares the commit the
`ref:` resolves to against the **gitlink** the parent records (`git ls-tree HEAD <path>`)
and warns when `.syscribe.toml` disagrees with `.gitmodules`. It is silent for non-submodule
peers (sibling checkouts, monorepo paths).
