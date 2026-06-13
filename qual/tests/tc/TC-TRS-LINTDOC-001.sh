tc_TRS_LINTDOC_001() {
    local F="$1"
    local B="$F/TC-TRS-LINTDOC-001"
    local M="$B/model"

    run_ld() {
        _flush_scenario
        SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0
        printf "  ▶ %s\n" "$SCENARIO_NAME"
        shift
        SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" lint-docs "$@" 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    }

    run_ld "W100 — unresolved Mermaid qname + W102 missing embed" "$B/docs/bad.md"
    grep -q "W100" <<<"$SCENARIO_OUTPUT" && grep -q "Ghost::Element" <<<"$SCENARIO_OUTPUT" && pass "W100 on Ghost::Element" || fail "no W100"
    grep -q "W102" <<<"$SCENARIO_OUTPUT" && grep -q "missing.svg" <<<"$SCENARIO_OUTPUT" && pass "W102 on missing.svg" || fail "no W102"

    run_ld "W101 — stale SVG sysml:ref" "$B/docs/diagram.svg"
    grep -q "W101" <<<"$SCENARIO_OUTPUT" && grep -q "Gone::Element" <<<"$SCENARIO_OUTPUT" && pass "W101 on Gone::Element" || fail "no W101"
    grep -q "Engine" <<<"$SCENARIO_OUTPUT" && fail "Engine (resolving ref) wrongly flagged" || pass "resolving sysml:ref clean"

    run_ld "resolving refs and prose qnames are clean" "$B/docs/good.md"
    [ -z "$SCENARIO_OUTPUT" ] && pass "no diagram findings on good.md" || fail "unexpected findings: $SCENARIO_OUTPUT"

    run_ld "--json shape" --json "$B/docs/bad.md"
    assert_output_contains "\"code\""
    assert_output_contains "\"ref\""
    assert_output_contains "W100"
}
