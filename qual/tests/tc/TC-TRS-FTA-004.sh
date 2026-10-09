tc_TRS_FTA_004() {
    local F="$1"; local B="$F/TC-TRS-FTA-004"

    run_scenario "gate cycle raises E960" "$B/cycle"
    assert_has_code "E960"

    run_scenario "gate arity raises E961" "$B/arity"
    assert_has_code "E961"

    run_scenario "value ranges raise E962 and E963" "$B/ranges"
    assert_has_code "E962"
    assert_has_code "E963"

    run_scenario "unreachable and stray nodes raise W960 and W961" "$B/reach"
    assert_has_code "W960"
    assert_has_code "W961"

    run_scenario "a well-formed tree raises none of the structural codes" "$B/clean"
    for c in E960 E961 E962 E963 E964 W960 W961 W962 W963 W964 W967; do
        assert_no_code "$c"
    done
}
