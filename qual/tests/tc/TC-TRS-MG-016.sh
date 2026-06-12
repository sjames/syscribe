tc_TRS_MG_016() {
    local F="$1"; local M="$F/TC-TRS-MG-013/clean"   # reuse the clean MagicGrid model
    local out rc W

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "magicgrid --svg prints a well-formed SVG with rows, pillars, and the SoI"
    out=$("$SYSCRIBE" -m "$M" magicgrid --svg 2>/dev/null || true)
    { printf '%s' "$out" | grep -qF "<svg" \
        && printf '%s' "$out" | grep -qF "</svg>" \
        && printf '%s' "$out" | grep -qF "Requirements" \
        && printf '%s' "$out" | grep -qF "Structure" \
        && printf '%s' "$out" | grep -qF "Station"; } \
        && pass "SVG shows pillars and the SoI" || fail "SVG missing root/pillars/SoI"

    _scn "exactly one <svg> root element"
    [ "$(printf '%s' "$out" | grep -cF '<svg')" -eq 1 ] \
        && pass "single <svg> root" || fail "expected exactly one <svg> root"

    _scn "-o writes the SVG to a file and stdout is empty"
    W=$(mktemp -d)
    out=$("$SYSCRIBE" -m "$M" magicgrid --svg -o "$W/grid.svg" 2>/dev/null || true)
    { [ -z "$out" ] && [ -s "$W/grid.svg" ] && grep -qF "<svg" "$W/grid.svg"; } \
        && pass "-o wrote the SVG, stdout empty" || fail "-o did not write the SVG cleanly"
    rm -rf "$W"

    _scn "the SVG works as a Diagram companion (no E402)"
    W=$(mktemp -d); cp -r "$M"/. "$W"/
    printf -- '---\ntype: Diagram\nname: MgGrid\nsvgMode: companion\n---\nMagicGrid companion diagram.\n' > "$W/MgGrid.md"
    "$SYSCRIBE" -m "$W" magicgrid --svg -o "$W/MgGrid.svg" >/dev/null 2>&1 || true
    out=$("$SYSCRIBE" -m "$W" validate 2>/dev/null || true)
    printf '%s' "$out" | grep -F "| E402 |" | grep -qF "MgGrid" \
        && fail "E402 raised — companion SVG not recognised" || pass "companion SVG satisfies E402"
    rm -rf "$W"

    _scn "a long cell label is word-wrapped, not truncated"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-MG-016/wrap" magicgrid --svg 2>/dev/null || true)
    { printf '%s' "$out" | grep -qF "energising" \
        && printf '%s' "$out" | grep -qF "connector" \
        && ! printf '%s' "$out" | grep -qF "…"; } \
        && pass "long label wrapped (all words present, no ellipsis)" || fail "long label not wrapped/truncated"
}
