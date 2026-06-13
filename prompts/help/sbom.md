# sbom — generate a Software Bill of Materials

## SYNOPSIS
    syscribe -m <root> sbom [--format cyclonedx|spdx] [--config <CONF>]
                            [--output <file>] [--include-tests] [--scope <qname>]

## DESCRIPTION
Generates an SBOM (§18) from the `implementedBy:` links on `Part`/`PartDef` elements (and,
with `--include-tests`, from `TestCase.sourceFile:`). Local paths become file components;
`<registry>:<package>@<version>` values become external package components with a PURL.
Locally-derived components carry external references back to the requirements the
implementing part satisfies (CycloneDX `externalReferences` / SPDX `GENERATED_FROM`).

Supported registries: `crates.io:`, `npm:`, `pypi:`, `maven:`, `nuget:`, `github:`.

## OPTIONS
    --format F          cyclonedx (CycloneDX 1.6, default) | spdx (SPDX 2.3).
    --config CONF       Project to a Configuration first (when a feature model is present).
    --output file       Write to a file instead of stdout.
    --include-tests     Include TestCase sourceFile entries as components.
    --scope qname       Restrict to a namespace subtree.

## SEE ALSO
    behavioral-coverage, links
