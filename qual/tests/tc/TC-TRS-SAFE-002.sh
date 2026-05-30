tc_TRS_SAFE_002() {
    local F="$1"
    local -a codes=(E805 E806 E825 E837 W801 W805 W806)
    for code in "${codes[@]}"; do
        run_scenario "trigger $code" "$F/TC-TRS-SAFE-002/$code"
        assert_has_code "$code"
    done
}
