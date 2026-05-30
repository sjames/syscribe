tc_TRS_VAL_003() {
    local F="$1"
    local -a codes=(W001 W002 W003 W004 W005 W006 W007 W300 W301 W302 W303 W304 W305)
    for code in "${codes[@]}"; do
        run_scenario "trigger $code" "$F/TC-TRS-VAL-003/$code"
        assert_has_code "$code"
    done
}
