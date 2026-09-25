# show — element details and documentation

## SYNOPSIS
    syscribe -m <root> show <qname|id> [--no-related]

## DESCRIPTION
Prints an element's frontmatter fields (type, status, integrity level, extRef,
domain, …), its inline features, and its documentation body. Accepts a qualified
name (Pkg::Sub::Name) or a stable id (REQ-*, TC-*, SG-*, …). User-defined links
(`links:`) appear as one `links.<type>` row per link type.

After the documentation body come the §3.10 annotations attached from other files:
one `## Documentation (<locale>)` section per locale variant (`locale:` +
`qualifiedName:`), and one `## Note: <name>` section per `about:` comment that lists
the element (with the comment's source file and locale). A `localeDocs` / `notes`
row in the field table summarises them.

For a package (`Package`/`LibraryPackage`/`Namespace`) — or any element that owns
child elements in the directory tree — a `## Members (N)` table lists its **direct**
children (Element = stable id else qualified name, Type, Name, Status), sorted by
qualified name and generated from the loaded model, never from the `_index.md` prose.
An empty package shows `## Members (0)` and `_(no members)_`. `--no-related` does not
suppress it.

Ends with a type-appropriate "Related:" footer suggesting the traceability
commands that answer the natural next questions about this same element —
`trace`/`who-verifies`/`impact`/`refs` for a `Requirement`; `impact`/
`connectivity`/`n2`/`refs` for an architecture element (`PartDef`/`Part`/…);
just `impact`/`refs` for everything else. Text-mode only.

## OPTIONS
    --no-related    Suppress the "Related:" footer (scripting/piping).

## EXAMPLES
    syscribe -m model/ show UAV::Avionics::FlightController
    syscribe -m model/ show REQ-UAV-NAV-001
    syscribe -m model/ show REQ-UAV-NAV-001 --no-related

## SEE ALSO
    links, refs, trace, find, ls, lint-docs
