tc_TRS_PARAM_003() {
    local F="$1"; local B="$F/TC-TRS-PARAM-003"

    run_scenario "binding 99 against range 1..=8 is out of range" "$B/over"
    assert_has_code "E205"

    run_scenario "binding 8 against range 1..=8 is in range" "$B/ok"
    assert_no_code "E205"
}
