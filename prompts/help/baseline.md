# baseline — release baselines (frozen, git-anchored release snapshots)

## SYNOPSIS
    syscribe -m <root> baseline create --tag <tag> [--name <n>] [--approver <a>]
                                       [--frozen-scope <sel>] [--id <BL-id>]
                                       [--allow-dirty] [--require-reviewed]
    syscribe -m <root> baseline verify <BL-id> | --all
    syscribe -m <root> baseline diff <BL-A> <BL-B> [--detail]
    syscribe -m <root> baseline list
    syscribe -m <root> baseline show <BL-id>

## DESCRIPTION
A `Baseline` (`BL-*`) is a named, dated, approved, frozen snapshot of a scope of the
model — the artifact an assessor points at. `create` walks the in-scope graph, hashes each
element's full canonical content (BLAKE3), aggregates a **seal**, captures the current git
commit, and writes both the `Baseline` element (`model/Baselines/<id>.md`) and a lean
JSON **manifest** (`<git-root>/baselines/<id>.manifest.json`).

Validation recomputes the in-scope aggregate and compares it to the seal, with
status-graded severity: `released` drift is **E520** (error), `approved` drift **W520**
(warning), `draft` silent, `superseded` skipped. A seal that disagrees with its manifest is
**E521**; an unresolved `supersedes:` is **E522**.

## SCOPE SELECTOR
`--frozen-scope` (and the `frozenScope:` field) select what is frozen. On the CLI it is a
single string of `;`-separated `key=value` clauses; multi-valued keys take a `,`-list:

    --frozen-scope "package=VehicleSystem::Powertrain;types=Requirement,TestCase;status=approved;tags=safety"

Omitting it ⇒ the whole model. `Baseline` elements are always excluded from a seal.

A `config=<Configuration>` clause freezes a **projected product-line variant**: the model is
first projected to that Configuration (or ad-hoc feature set) — dropping every element
inactive under it — and the remaining filters apply over the variant. Drift-checking and
`verify` re-project, so a change to the variant's active content (or to the Configuration's
selections) is detected. Example: `--frozen-scope "config=CONF-ABS;status=approved"`.

## OUTPUT LOCATIONS
By default the element goes to `model/Baselines/<id>.md` and the manifest to
`<git-root>/baselines/<id>.manifest.json`. A `[baselines]` table in `.syscribe.toml`
redirects both:

    [baselines]
    element_dir  = "Releases"    # under the model root (must stay within the model tree)
    manifest_dir = "evidence"    # under the git root (may be anywhere)

The manifest path is self-recorded in the element's `seal`, so `verify` and drift-checking
resolve it wherever it lives.

## GIT
`create` records the current `HEAD` as `gitCommit` and expects a **clean** working tree
(`--allow-dirty` to override). It does **not** create the git tag — tag the release
yourself; `verify` checks that `gitTag` resolves to `gitCommit` when the tag exists.
`diff --detail` reconstructs field/body changes via `git show`.

## EXAMPLES
    syscribe -m model/ baseline create --tag REL-2026-07 --approver "J. Roe"
    syscribe -m model/ baseline verify --all
    syscribe -m model/ baseline diff BL-2026-06 BL-2026-07 --detail
    syscribe -m model/ validate --deny E520   # already fatal; gates released drift

## SEE ALSO
    suspect, validate, trace
