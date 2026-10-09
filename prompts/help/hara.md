# hara — HARA ASIL matrix (heat table)

## SYNOPSIS
    syscribe -m <root> hara matrix [--format md|html|json]

## DESCRIPTION
`hara matrix` renders the ISO 26262-3 Table 4 determination as a heat table: rows
are severity x exposure (S1/E1 .. S3/E4), columns are controllability (C1..C3) and
each cell is the derived ASIL (QM, A, B, C, D) coloured by level. Every
HazardousEvent is placed in the cell its severity/exposure/controllability select,
and every SafetyGoal is placed (marked "goal") in the cell of its highest-ASIL
linked hazardous event. Events with an S0/E0/C0 class (always QM) or with an
unparseable class are listed under "Not placed" rather than dropped.

## OPTIONS
    matrix                 Render the matrix (the only sub-command).
    --format md|html|json  Markdown (default), a standalone self-contained HTML
                           page (inline CSS, no scripts, no network), or JSON
                           (rows, cols, cells {label, tone, elements}, unplaced).

## EXAMPLES
    syscribe -m model_sil/ hara matrix
    syscribe -m model_sil/ hara matrix --format json
    syscribe -m model_auto/ hara matrix --format html

## NOTES
The same heat views exist for FMEA (`fmea report --format ...`, severity x
occurrence with RPN colouring) and for TARA (`cyber-risk --format ...`, impact x
feasibility under the configured [cyber] method). The ASIL arithmetic is shared
with the validator (W811/W812).

## SEE ALSO
    fmea, cyber-risk, validate, spec safety
