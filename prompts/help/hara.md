# hara — HARA ASIL matrix (heat table)

## SYNOPSIS
    syscribe -m <root> hara matrix [--format md|html|json]
    syscribe -m <root> hara trace [<subject>] [--format dot|mermaid|plantuml|svg]

## DESCRIPTION
`hara matrix` renders the ISO 26262-3 Table 4 determination as a heat table: rows
are severity x exposure (S1/E1 .. S3/E4), columns are controllability (C1..C3) and
each cell is the derived ASIL (QM, A, B, C, D) coloured by level. Every
HazardousEvent is placed in the cell its severity/exposure/controllability select,
and every SafetyGoal is placed (marked "goal") in the cell of its highest-ASIL
linked hazardous event. Events with an S0/E0/C0 class (always QM) or with an
unparseable class are listed under "Not placed" rather than dropped.

`hara trace` draws the hazard-to-test graph (the Traceability diagram kind):
HazardousEvent <- SafetyGoal <- Requirements <- TestCases, with each goal's fault tree
and arguments. A test case shows its verdict from the results sidecar (`unknown`
without one); nodes are coloured by the roll-up of what is below them and gaps are red
badges naming the finding (`W002`/`W003 no test`, `W305 no integration test`,
`W300 unsatisfied`, `no requirement`, `no goal`). The subject is a SafetyGoal,
HazardousEvent, Requirement or Package (default: the whole model); a wrong one exits 1.

## OPTIONS
    matrix                 Render the matrix.
    trace [<subject>]      Draw the hazard-to-test graph; --format dot (default),
                           mermaid, plantuml or svg.
    --format md|html|json  Markdown (default), a standalone self-contained HTML
                           page (inline CSS, no scripts, no network), or JSON
                           (rows, cols, cells {label, tone, elements}, unplaced).

## EXAMPLES
    syscribe -m model_sil/ hara matrix
    syscribe -m model_sil/ hara matrix --format json
    syscribe -m model_auto/ hara matrix --format html
    syscribe -m model_auto/ hara trace SG-ENG-001 --format mermaid

## NOTES
The same heat views exist for FMEA (`fmea report --format ...`, severity x
occurrence with RPN colouring) and for TARA (`cyber-risk --format ...`, impact x
feasibility under the configured [cyber] method). The ASIL arithmetic is shared
with the validator (W811/W812).

## SEE ALSO
    fmea, cyber-risk, validate, spec safety
