tc_TRS_VAL_004() {
    local F="$1"
    run_scenario "E841: derivedFromSafetyGoal element missing integrity level" "$F/TC-TRS-VAL-004/E841"
    assert_has_code "E841"

    run_scenario "E842: derivedFrom element missing integrity level" "$F/TC-TRS-VAL-004/E842"
    assert_has_code "E842"

    run_scenario "E843: satisfies element missing integrity level" "$F/TC-TRS-VAL-004/E843"
    assert_has_code "E843"

    run_scenario "W808: integrity level lower than source without breakdownAdr" "$F/TC-TRS-VAL-004/W808"
    assert_has_code "W808"
}
