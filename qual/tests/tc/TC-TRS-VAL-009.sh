tc_TRS_VAL_009() {
    local F="$1"
    local -a codes=(E500 E501 E502 E503 W500 W501 W502 W600 W601)
    for code in "${codes[@]}"; do
        run_scenario "trigger $code" "$F/TC-TRS-VAL-009/$code"
        assert_has_code "$code"
    done
}
