tc_TRS_QNAME_003() {
    local F="$1"
    run_scenario "name: in frontmatter does not replace the filename stem" "$F/TC-TRS-QNAME-003/element-name-override"
    assert_no_code "E005"
    assert_exit_zero
    assert_output_contains "'Engine' is defined"
    grep -qF "'InternalCombustionEngine'" <<<"$SCENARIO_OUTPUT" \
        && fail "label-derived qname InternalCombustionEngine reported" || pass "no label-derived qname InternalCombustionEngine"
}
