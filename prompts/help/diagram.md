# diagram — diagram authoring workflow

## SYNOPSIS
    syscribe -m <root> diagram <list|measure|render|compose|layout|seq|req> [args]

## DESCRIPTION
Drives the companion-SVG diagram workflow: `diagram list` enumerates candidate
elements as JSON, `diagram measure` emits element sizes, and `diagram compose`
renders a *.layout.json (or a Diagram element's `expose:` list) into an SVG.
(Layout files are ephemeral inputs and must not be committed — name them
*.layout.json so .gitignore excludes them.)

An element that does not exist is an error, never a warning: when a `measure`
qname, a placed qname in a layout/placement file, an `expose:` entry, or a
`render`/`seq`/`req` target does not resolve, the command prints
`error: element '<qname>' not found` on stderr for each, writes nothing to
stdout, and exits 1.

## EXAMPLES
    syscribe -m model/ diagram list --type PartDef
    syscribe -m model/ diagram measure UAV::Airframe,UAV::Avionics::FlightController
    syscribe -m model/ diagram compose Views::SystemArchitectureView --emit-placement

## SEE ALSO
    render
