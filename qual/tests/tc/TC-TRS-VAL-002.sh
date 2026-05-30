tc_TRS_VAL_002() {
    local F="$1"
    local -a codes=(E101 E102 E103 E104 E105 E106 E310 E311 E312 E313 E314 E315)
    for code in "${codes[@]}"; do
        run_scenario "trigger $code" "$F/TC-TRS-VAL-002/$code"
        assert_has_code "$code"
    done
}
