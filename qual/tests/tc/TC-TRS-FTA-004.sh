tc_TRS_FTA_004() {
    local F="$1"; local B="$F/TC-TRS-FTA-004"

    run_scenario "gate cycle raises E980" "$B/cycle"
    assert_has_code "E980"

    run_scenario "gate arity raises E981" "$B/arity"
    assert_has_code "E981"

    run_scenario "value ranges raise E982 and E983" "$B/ranges"
    assert_has_code "E982"
    assert_has_code "E983"

    run_scenario "unreachable and stray nodes raise W980 and W981" "$B/reach"
    assert_has_code "W980"
    assert_has_code "W981"

    run_scenario "a well-formed tree raises none of the structural codes" "$B/clean"
    for c in E980 E981 E982 E983 E984 W980 W981 W982 W983 W984 W987; do
        assert_no_code "$c"
    done
}
