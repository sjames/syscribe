tc_TRS_DIAG_001() {
    local F="$1"
    # Error codes — simple loop: one fixture per code
    local -a error_codes=(E400 E401 E402)
    for code in "${error_codes[@]}"; do
        run_scenario "trigger $code" "$F/TC-TRS-DIAG-001/$code"
        assert_has_code "$code"
    done

    # Warning codes — simple loop for codes with single fixture directories
    local -a warn_codes=(W400 W401 W402 W403 W404 W406 W407 W408 W409 W410 W411 W412)
    for code in "${warn_codes[@]}"; do
        run_scenario "trigger $code" "$F/TC-TRS-DIAG-001/$code"
        assert_has_code "$code"
    done

    # W405 has two sub-scenarios: companion mode missing <img, and inline mode missing ```svg
    run_scenario "trigger W405 (companion mode, no img tag)" "$F/TC-TRS-DIAG-001/W405a"
    assert_has_code "W405"

    run_scenario "trigger W405 (inline mode, no svg block)" "$F/TC-TRS-DIAG-001/W405b"
    assert_has_code "W405"
}
