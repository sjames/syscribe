tc_TRS_VAL_008() {
    local F="$1"
    # Loop over enum-validation error codes
    local -a codes=(E019 E020 E021 E022 W703)
    for code in "${codes[@]}"; do
        run_scenario "trigger $code" "$F/TC-TRS-VAL-008/$code"
        assert_has_code "$code"
    done

    run_scenario "W008: file with valid frontmatter but no type: field" "$F/TC-TRS-VAL-008/W008"
    assert_has_code "W008"

    run_scenario "W701: Requirement with asilLevel B/C/D and no verificationMethod" "$F/TC-TRS-VAL-008/W701"
    assert_has_code "W701"

    run_scenario "W702: ASIL-D Requirement with active TestCase but none at L5" "$F/TC-TRS-VAL-008/W702"
    assert_has_code "W702"
}
