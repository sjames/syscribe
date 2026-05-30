tc_TRS_SAFE_001() {
    local F="$1"
    local -a codes=(E800 E801 E802 E803 E804 E833 E834 E835 E836 W800)
    for code in "${codes[@]}"; do
        run_scenario "trigger $code" "$F/TC-TRS-SAFE-001/$code"
        assert_has_code "$code"
    done
}
