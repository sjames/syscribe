tc_TRS_SAFE_004() {
    local F="$1"
    local -a codes=(E815 E816 E817 E818 E819 E820 E821 E822 E823 E824 E827 E828 E829 E830 E831 E832 W802 W803 W804 W807)
    for code in "${codes[@]}"; do
        run_scenario "trigger $code" "$F/TC-TRS-SAFE-004/$code"
        assert_has_code "$code"
    done
}
