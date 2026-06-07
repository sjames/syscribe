tc_TRS_FMA_011() {
    local F="$1"; local V="$F/TC-TRS-FMA-005/void"

    # Scenario: --prove emits DIMACS + DRAT for a void model.
    SCENARIO_NAME="--prove emits DIMACS + DRAT for a void model"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local dir; dir=$(mktemp -d)
    "$SYSCRIBE" -m "$V" feature-check --deep --prove "$dir" >/dev/null 2>&1 || true
    local ndrat ncnf
    ndrat=$(find "$dir" -name '*.drat' -size +0c 2>/dev/null | wc -l)
    ncnf=$(find "$dir" \( -name '*.cnf' -o -name '*.dimacs' \) -size +0c 2>/dev/null | wc -l)
    [ "$ndrat" -ge 1 ] && pass "a non-empty DRAT proof was written" || fail "no DRAT proof written"
    [ "$ncnf" -ge 1 ] && pass "a non-empty DIMACS CNF was written" || fail "no DIMACS CNF written"
    rm -rf "$dir"

    # Scenario: no proof files without --prove.
    SCENARIO_NAME="no proof files without --prove"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local dir2; dir2=$(mktemp -d)
    ( cd "$dir2" && "$SYSCRIBE" -m "$V" feature-check --deep >/dev/null 2>&1 || true )
    local n; n=$(find "$dir2" -type f 2>/dev/null | wc -l)
    [ "$n" -eq 0 ] && pass "no proof files emitted without --prove" || fail "unexpected files written without --prove (${n})"
    rm -rf "$dir2"
}
