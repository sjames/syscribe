tc_TRS_TARA_001() {
    local F="$1"
    local -a codes=(E940 E941 W905)
    for code in "${codes[@]}"; do
        run_scenario "trigger $code" "$F/TC-TRS-TARA-001/$code"
        assert_has_code "$code"
    done
}
