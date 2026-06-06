tc_TRS_VAL_010() {
    local F="$1"

    # All supported languages + a generic file resolve → no W009.
    run_scenario "resolving functions across all languages produce no W009" "$F/TC-TRS-VAL-010/clean"
    assert_no_code "W009"

    # Renamed Rust function + missing generic test → two W009 findings.
    run_scenario "renamed/missing tests produce W009" "$F/TC-TRS-VAL-010/W009"
    assert_has_code "W009"
    assert_count "W009" 2
}
