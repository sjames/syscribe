tc_TRS_SAFE_003() {
    local F="$1"
    local -a codes=(E807 E808 E809 E810 E811 E812 E813 E814 E826)
    for code in "${codes[@]}"; do
        run_scenario "trigger $code" "$F/TC-TRS-SAFE-003/$code"
        assert_has_code "$code"
    done
}
