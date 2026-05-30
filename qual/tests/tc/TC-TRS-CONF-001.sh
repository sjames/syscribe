tc_TRS_CONF_001() {
    local F="$1"
    local -a codes=(E200 E201 E209)
    for code in "${codes[@]}"; do
        run_scenario "trigger $code" "$F/TC-TRS-CONF-001/$code"
        assert_has_code "$code"
    done
}
