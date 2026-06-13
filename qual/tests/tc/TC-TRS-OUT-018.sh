tc_TRS_OUT_018() {
    local F="$1"
    local M="$F/TC-TRS-OUT-018/cov"

    run_bc() {
        _flush_scenario
        SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0
        printf "  ▶ %s\n" "$SCENARIO_NAME"
        shift
        SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$1" behavioral-coverage "${@:2}" 2>/dev/null); SCENARIO_EXIT=$?
    }

    run_bc "default coverage (paths 1 and 4)" "$M"
    grep -qE "ActSrc .*✓" <<<"$SCENARIO_OUTPUT" && pass "ActSrc covered (path 1)" || fail "ActSrc not covered"
    grep -qE "ActAlloc .*✓" <<<"$SCENARIO_OUTPUT" && pass "ActAlloc covered (path 4)" || fail "ActAlloc not covered"
    grep -qE "ActUncovered .*✗" <<<"$SCENARIO_OUTPUT" && pass "ActUncovered uncovered" || fail "ActUncovered wrong"
    grep -qE "ActPlanned .*✗" <<<"$SCENARIO_OUTPUT" && pass "ActPlanned uncovered by default (draft test)" || fail "ActPlanned wrong"
    assert_output_contains "2 / 4"

    run_bc "--include-planned surfaces planned coverage" "$M" --include-planned
    grep -q "ActPlanned.*TC-BCOV-PLAN-001" <<<"$SCENARIO_OUTPUT" && pass "planned column shows the draft test" || fail "planned column missing"

    run_bc "--uncovered-only keeps the true percentage" "$M" --uncovered-only
    assert_output_contains "2 / 4"
    grep -q "ActSrc" <<<"$SCENARIO_OUTPUT" && fail "covered element leaked past --uncovered-only" || pass "covered elements hidden"

    run_bc "json schema" "$M" --format json
    assert_output_contains "\"coverage_pct\""
    assert_output_contains "\"coveredBy\""

    # Demo model achieves >50% out of the box
    _flush_scenario
    SCENARIO_NAME="demo model >50% behavioral coverage"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$REPO_ROOT/model" behavioral-coverage 2>/dev/null); SCENARIO_EXIT=$?
    assert_output_contains "3 / 5"
}
