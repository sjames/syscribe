tc_TRS_TRACE_009() {
    local F="$1"

    run_scenario "supertype cycle produces E016" "$F/TC-TRS-TRACE-009/supertype-cycle"
    assert_has_code "E016"
    assert_no_code "E017"
    assert_no_code "E018"
    assert_exit_nonzero

    run_scenario "derivedFrom cycle produces E017" "$F/TC-TRS-TRACE-009/derived-from-cycle"
    assert_has_code "E017"
    assert_no_code "E016"
    assert_no_code "E018"
    assert_exit_nonzero

    run_scenario "subsets cycle produces E018" "$F/TC-TRS-TRACE-009/subsets-cycle"
    assert_has_code "E018"
    assert_no_code "E016"
    assert_no_code "E017"
    assert_exit_nonzero

    run_scenario "acyclic model produces no cycle errors" "$F/TC-TRS-TRACE-009/no-cycle"
    assert_no_code "E016"
    assert_no_code "E017"
    assert_no_code "E018"
    assert_exit_zero
}
