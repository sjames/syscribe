# export-reqif — export Requirements as a ReqIF 1.2 document

## SYNOPSIS
    syscribe -m <root> export-reqif [--output <file>] [--scope <qname>]
                                    [--config <CONF>] [--include-tests] [--zip]

## DESCRIPTION
Exports native `Requirement` elements (and their packages) as a ReqIF 1.2 XML document
(§21) for import into DOORS Next / Jama / Polarion / PTC. Each requirement becomes a
`SPEC-OBJECT` (id/name/status/sil/asil/domain attributes + an XHTML `DESC` from the body);
packages become a nested `SPEC-HIERARCHY`; `derivedFrom:` links become `DERIVED_FROM`
`SPEC-RELATION`s. Export-only — importing ReqIF is out of scope.

## OPTIONS
    --output file     Write to <file>.reqif (or <file>.reqifz with --zip); default stdout.
    --scope qname     Export only requirements in this namespace subtree.
    --config CONF     Project to a Configuration first (export active requirements only).
    --include-tests   Also emit TestCases as SPEC-OBJECTs + VERIFIED_BY relations.
    --zip             Package as a .reqifz ZIP (stored content.reqif entry).

## SEE ALSO
    sbom, links, trace
