tc_TRS_TYPE_019() {
    local F="$1"
    local BASE="$F/TC-TRS-TYPE-019"

    run_scenario "well-formed TradeStudy is clean" "$BASE/clean"
    for c in E869 E870 E871 E872 E873 E874 E875 E876 E877 W061 W062 W063 W064; do assert_no_code "$c"; done

    run_scenario "E869 — missing scores" "$BASE/e869_missing";      assert_has_code "E869"
    run_scenario "E870 — bad id" "$BASE/e870_id";                   assert_has_code "E870"
    run_scenario "E871 — criterion missing field" "$BASE/e871_criteria"; assert_has_code "E871"
    run_scenario "E872 — weight out of range" "$BASE/e872_weight";  assert_has_code "E872"
    run_scenario "E873 — bad direction" "$BASE/e873_direction";     assert_has_code "E873"
    run_scenario "E874 — empty alternatives" "$BASE/e874_empty";    assert_has_code "E874"
    run_scenario "E875 — alternative missing name" "$BASE/e875_altname"; assert_has_code "E875"
    run_scenario "E876 — unknown alternative" "$BASE/e876_unknown"; assert_has_code "E876"
    run_scenario "E877 — non-numeric score" "$BASE/e877_score";     assert_has_code "E877"
    run_scenario "W061 — complete without decision" "$BASE/w061_nodecision"; assert_has_code "W061"
    run_scenario "W063 — incomplete matrix" "$BASE/w063_incomplete"; assert_has_code "W063"

    # CLI: trade-study detail (ranked table)
    _flush_scenario
    SCENARIO_NAME="trade-study detail ranked table"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/clean" trade-study TRD-COMM-001 2>/dev/null); SCENARIO_EXIT=$?
    assert_output_contains "Rank"
    assert_output_contains "#1"

    # CLI: template
    _flush_scenario
    SCENARIO_NAME="template TradeStudy skeleton"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/clean" template TradeStudy 2>/dev/null); SCENARIO_EXIT=$?
    assert_output_contains "type: TradeStudy"
}
