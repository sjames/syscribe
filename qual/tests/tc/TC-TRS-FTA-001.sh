tc_TRS_FTA_001() {
    local F="$1"
    local -a codes=(E900 E901 E902 E903 E904 E905 E906 E907 E908 E909 W900 W901)
    for code in "${codes[@]}"; do
        run_scenario "trigger $code" "$F/TC-TRS-FTA-001/$code"
        assert_has_code "$code"
    done
}
