tc_TRS_FMEA_001() {
    local F="$1"
    local -a codes=(E911 E912 E913 E914 W902 W903 W904)
    for code in "${codes[@]}"; do
        run_scenario "trigger $code" "$F/TC-TRS-FMEA-001/$code"
        assert_has_code "$code"
    done
}
