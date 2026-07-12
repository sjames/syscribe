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
