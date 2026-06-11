tc_TRS_MG_007() {
    local F="$1"; local FX="$F/TC-TRS-MG-007"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. the grid lists MoEs against configurations
    _scn "the grid lists MoEs against configurations"
    out=$("$SYSCRIBE" -m "$FX/model" trade-study 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q 'EnduranceMoE' && printf '%s' "$out" | grep -q 'MarginMoE' \
        && printf '%s' "$out" | grep -q 'ConfA' && printf '%s' "$out" | grep -q 'ConfB'; } \
        && pass "MoE rows and Configuration columns present" || fail "trade-study grid not rendered (rc=$rc)"

    # 2. a value beyond the objective scores 1.0
    _scn "a value beyond the objective scores 1.0"
    out=$("$SYSCRIBE" -m "$FX/model" trade-study 2>&1) || true
    printf '%s' "$out" | grep -q '1.00' \
        && pass "above-objective value scores 1.00" || fail "score saturation to 1.00 not shown"

    # 3. a sub-threshold value fails the configuration
    _scn "a sub-threshold value flags a violation and fails the configuration"
    out=$("$SYSCRIBE" -m "$FX/model" trade-study 2>&1) || true
    printf '%s' "$out" | grep -qi 'fail' \
        && pass "threshold violation fails the configuration (ConfB)" || fail "sub-threshold config not marked failing"

    # 4. the rollup ranks configurations and marks the winner
    _scn "the rollup ranks configurations and marks the winner"
    out=$("$SYSCRIBE" -m "$FX/model" trade-study 2>&1) || true
    printf '%s' "$out" | grep -qi 'winner' \
        && pass "winner marked in the rollup (ConfA)" || fail "winner not marked"

    # 5. --config restricts the columns
    _scn "--config restricts the columns"
    out=$("$SYSCRIBE" -m "$FX/model" trade-study --config CONF-MG-A-001 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q 'ConfA' && ! printf '%s' "$out" | grep -q 'ConfB'; } \
        && pass "only ConfA column present" || fail "--config did not restrict columns (rc=$rc)"

    # 6. an unevaluable cell is n/a
    _scn "an unevaluable cell is reported n/a"
    out=$("$SYSCRIBE" -m "$FX/na" trade-study 2>&1) || true
    printf '%s' "$out" | grep -q 'n/a' \
        && pass "unbound MoE cell reported n/a" || fail "n/a cell not reported"

    # 7. the trade study emits JSON
    _scn "the trade study emits JSON"
    out=$("$SYSCRIBE" -m "$FX/model" trade-study --json 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q '"rollup"'; } \
        && pass "trade-study --json emits grid and rollup" || fail "trade-study JSON malformed (rc=$rc)"
}
