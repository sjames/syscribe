# lint-docs — scan external Markdown/SVG for unresolvable model references

## SYNOPSIS
    syscribe -m <root> lint-docs <path>... [--json]

## DESCRIPTION
Scans `.md` and `.svg` files (or directories, recursively) for references to model
elements that do not resolve in the loaded model: stable-ID tokens in prose, qualified
names inside Mermaid blocks, SVG `sysml:ref` manifests, and local image/diagram embed
paths.

It also flags a package `_index.md` that hand-enumerates its own members (`W103`,
advisory): membership is generated from the directory by `show <package>`, the web UI
and `export-html`, so a hand-maintained list only drifts.

Exits non-zero if any unresolvable references are found (`W099`–`W102`), enabling CI
gating. `W103` alone never makes it exit non-zero.

## OPTIONS
    <path>...    Files or directories to scan (.md and .svg).
    --json       Emit findings as a JSON array of {file, line, code, token|ref|path|detail}
                 (`detail` carries the W103 message).

## EXAMPLES
    syscribe -m model/ lint-docs docs/
    syscribe -m model/ lint-docs docs/ --json
    syscribe -m model/ lint-docs docs/architecture.svg

## NOTES
Findings (all gateable like other warnings, e.g. `--deny W100`):

- **W099** — an unresolvable stable-ID token (`REQ-*`, `TC-*`, …) in prose.
- **W100** — a qualified name (`A::B::C`) inside a ` ```mermaid ` block that does not
  resolve. (Qualified names in *prose* are deliberately not resolved — false-positive prone.)
- **W101** — an SVG `sysml:ref="…"` that does not resolve. An SVG with no `sysml:ref`
  attributes is treated as opaque (no findings).
- **W102** — a local image/diagram embed path (`![](path)`, `<img src>`) that does not
  exist. Remote URIs (`https://…`) are accepted as external.
- **W103** (advisory) — a package's own `_index.md` whose body (frontmatter excluded)
  mentions three or more distinct stable ids of that package's *direct* members. The
  message names the count and points at `syscribe show <package>`; state the package's
  purpose and scope in `_index.md` instead of listing its members. Ids of other
  packages' elements, fewer than three members, and files that are not a package's
  `_index.md` never raise it. Directory scans apply it too.

## SEE ALSO
    validate, check-ref, refs, show
