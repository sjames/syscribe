tc_TRS_MG_003() {
    local F="$1"; local FX="$F/TC-TRS-MG-003"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a classified element appears in its grid cell
    _scn "a classified element appears in its grid cell"
    out=$("$SYSCRIBE" -m "$FX/cells" magicgrid 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q 'DriveUseCase' && printf '%s' "$out" | grep -q 'B2'; } \
        && pass "DriveUseCase listed in cell B2" || fail "classified element not shown in grid (rc=$rc)"

    # 2. an invalid coordinate raises MG020 under the gate
    _scn "an invalid mg_cell coordinate raises MG020 under the gate"
    out=$("$SYSCRIBE" -m "$FX/mg020" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    { [ "$rc" -ne 0 ] && printf '%s' "$out" | grep -q 'MG020'; } \
        && pass "MG020 raised, non-zero exit" || fail "MG020 not raised under gate (rc=$rc)"

    # 3. a type/column mismatch raises MG021 under the gate
    _scn "a type/column mismatch raises MG021 under the gate"
    out=$("$SYSCRIBE" -m "$FX/mg021" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    { [ "$rc" -ne 0 ] && printf '%s' "$out" | grep -q 'MG021'; } \
        && pass "MG021 raised, non-zero exit" || fail "MG021 not raised under gate (rc=$rc)"

    # 4. the grid report flags empty cells
    _scn "the grid report flags empty cells"
    out=$("$SYSCRIBE" -m "$FX/cells" magicgrid 2>&1) || true
    printf '%s' "$out" | grep -qiE 'empty cells?' \
        && pass "empty cells reported" || fail "empty cells not flagged in grid report"

    # 5. the grid report emits JSON
    _scn "the grid report emits JSON"
    out=$("$SYSCRIBE" -m "$FX/cells" magicgrid --json 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q '"B2"'; } \
        && pass "magicgrid --json emits a grid structure" || fail "magicgrid --json malformed (rc=$rc)"

    # 6. mg_cell is inert without the gate
    _scn "mg_cell is inert without the magicgrid profile"
    out=$("$SYSCRIBE" -m "$FX/mg020" validate 2>&1) || true
    printf '%s' "$out" | grep -q 'MG020' \
        && fail "MG020 raised without the gate" || pass "no MG02x finding without the gate"
}
