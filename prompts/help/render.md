# render — print a Diagram element's embedded diagram with element links injected

## SYNOPSIS
    syscribe -m <root> render <diagram-path>

## DESCRIPTION
Prints the diagram embedded in a Diagram element's body to stdout, with links to
the model elements it shows injected. <diagram-path> is the element's file path
(or a path suffix of it), not a qualified name.

- `diagramKind: Mermaid` — the ```mermaid block, with a `click <node> href`
  directive added for every node that names a model element.
- any other `diagramKind` — the embedded ```svg block, with `<a href>` wrappers
  added around shapes that reference model elements.

It does not lay out or generate a diagram: to generate SVG from the model graph
use `diagram` (render/compose/seq/req); for PlantUML sources use `plantuml`.

## EXIT CODES
    0  printed    1  no element at the path, no `diagramKind:`, or the expected
                     ```mermaid / ```svg block is missing

## EXAMPLES
    syscribe -m model/ render model/Diagrams/RequirementTraceMermaid.md

## SEE ALSO
    diagram
