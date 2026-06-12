tc_TRS_MG_015() {
    local F="$1"; local M="$F/TC-TRS-MG-013/clean"   # reuse the clean MagicGrid model
    local out
    out=$("$SYSCRIBE" -m "$M" magicgrid 2>/dev/null || true)

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "the report contains a grid table header with the four pillars"
    printf '%s' "$out" | grep -qF "| Row |" \
        && printf '%s' "$out" | grep -F "| Row |" | grep -qF "Requirements" \
        && printf '%s' "$out" | grep -F "| Row |" | grep -qF "Behavior" \
        && printf '%s' "$out" | grep -F "| Row |" | grep -qF "Structure" \
        && printf '%s' "$out" | grep -F "| Row |" | grep -qF "Parameters" \
        && pass "grid header lists the four pillars" || fail "no grid header row with the four pillars"

    _scn "each B/W/S row appears as a grid row"
    { printf '%s' "$out" | grep -qE '^\| \*\*B\*\*' \
        && printf '%s' "$out" | grep -qE '^\| \*\*W\*\*' \
        && printf '%s' "$out" | grep -qE '^\| \*\*S\*\*'; } \
        && pass "B/W/S grid rows present" || fail "missing a B/W/S grid row"

    _scn "the B3 System of Interest is marked in the grid"
    printf '%s' "$out" | grep -qF "◆" \
        && pass "SoI marker present" || fail "no SoI marker in the grid"

    _scn "the populated-cell count is reported"
    printf '%s' "$out" | grep -qE '3/12 cells populated' \
        && pass "populated count 3/12 shown" || fail "no populated-cell count"

    _scn "the per-cell element detail is retained"
    printf '%s' "$out" | grep -qF "ServeUseCase" \
        && pass "per-cell detail still present" || fail "per-cell detail lost"
}
