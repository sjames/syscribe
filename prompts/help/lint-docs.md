# lint-docs — scan external Markdown/SVG for unresolvable model references

## SYNOPSIS
    syscribe -m <root> lint-docs <path>... [--json]

## DESCRIPTION
Scans `.md` and `.svg` files (or directories, recursively) for references to model
elements that do not resolve in the loaded model: stable-ID tokens in prose, qualified
names inside Mermaid blocks, SVG `sysml:ref` manifests, and local image/diagram embed
paths.

Exits non-zero if any unresolvable references are found, enabling CI gating.

## OPTIONS
    <path>...    Files or directories to scan (.md and .svg).
    --json       Emit findings as a JSON array of {file, line, code, token|ref|path}.

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

## SEE ALSO
    validate, check-ref, refs
