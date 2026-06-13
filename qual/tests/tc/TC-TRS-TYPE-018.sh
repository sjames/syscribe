tc_TRS_TYPE_018() {
    local F="$1"
    local BASE="$F/TC-TRS-TYPE-018"

    run_scenario "well-formed ReviewRecord is clean" "$BASE/clean"
    for c in E700 E701 E702 E703 E704 E705 W700; do assert_no_code "$c"; done

    run_scenario "E700 — missing required fields" "$BASE/e700_missing"
    assert_has_code "E700"

    run_scenario "E701 — bad id pattern" "$BASE/e701_id"
    assert_has_code "E701"

    run_scenario "E702 — bad status" "$BASE/e702_status"
    assert_has_code "E702"

    run_scenario "E703 — bad reviewType" "$BASE/e703_type"
    assert_has_code "E703"

    run_scenario "E704 — unresolved reviews entry" "$BASE/e704_unresolved"
    assert_has_code "E704"

    run_scenario "E705 — bad item disposition" "$BASE/e705_disposition"
    assert_has_code "E705"

    run_scenario "W700 — closed review with open item" "$BASE/w700_open"
    assert_has_code "W700"

    run_scenario "W704 — uncovered requirement" "$BASE/w704_coverage"
    assert_has_code "W704"

    # CLI: reviews list
    _flush_scenario
    SCENARIO_NAME="reviews lists the record"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/clean" reviews 2>/dev/null); SCENARIO_EXIT=$?
    assert_output_contains "RR-SW-001"

    # CLI: reviews --coverage
    _flush_scenario
    SCENARIO_NAME="reviews --coverage shows uncovered requirement"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/w704_coverage" reviews --coverage 2>/dev/null); SCENARIO_EXIT=$?
    assert_output_contains "REQ-COV-BB-001"

    # CLI: review detail
    _flush_scenario
    SCENARIO_NAME="review shows detail"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/clean" review RR-SW-001 2>/dev/null); SCENARIO_EXIT=$?
    assert_output_contains "design_review"

    # CLI: template
    _flush_scenario
    SCENARIO_NAME="template ReviewRecord skeleton"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BASE/clean" template ReviewRecord 2>/dev/null); SCENARIO_EXIT=$?
    assert_output_contains "type: ReviewRecord"
}
