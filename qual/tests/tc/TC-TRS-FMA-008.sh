tc_TRS_FMA_008() {
    local F="$1"; local M="$F/TC-TRS-FMA-008/model"
    local NOFM="$F/TC-TRS-FM-001/no-fm"

    # Scenario: forced + free features from a partial selection {A: true}, A requires B.
    SCENARIO_NAME="configure reports forced and free features"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local j; j=$("$SYSCRIBE" -m "$M" configure CONF-FMA8-PARTIAL-001 --json 2>/dev/null || true)
    printf '%s' "$j" | jq -e '.satisfiable == true' >/dev/null 2>&1 \
        && pass "partial selection is satisfiable" || fail "partial selection not satisfiable"
    printf '%s' "$j" | jq -e '.forcedTrue | index("Features::B")' >/dev/null 2>&1 \
        && pass "B is forced-true (A requires B)" || fail "B not reported forced-true"
    printf '%s' "$j" | jq -e '.free | index("Features::C")' >/dev/null 2>&1 \
        && pass "C is free" || fail "C not reported free"

    # Scenario: contradictory partial selection {A: true, B: false}.
    SCENARIO_NAME="contradictory partial selection is unsatisfiable, exit 1"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local jb; jb=$("$SYSCRIBE" -m "$M" configure CONF-FMA8-BAD-001 --json 2>/dev/null || true)
    printf '%s' "$jb" | jq -e '.satisfiable == false' >/dev/null 2>&1 \
        && pass "contradiction reported unsatisfiable" || fail "contradiction not flagged"
    "$SYSCRIBE" -m "$M" configure CONF-FMA8-BAD-001 >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -eq 1 ] && pass "contradictory configure exits 1" || fail "exit ${ec} (expected 1)"

    # Scenario: dormant with no feature model.
    SCENARIO_NAME="dormant with no feature model"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out; out=$("$SYSCRIBE" -m "$NOFM" configure CONF-X --json 2>/dev/null || true)
    "$SYSCRIBE" -m "$NOFM" configure CONF-X >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -eq 0 ] && pass "dormant configure exits 0" || fail "dormant exit ${ec}"
}
