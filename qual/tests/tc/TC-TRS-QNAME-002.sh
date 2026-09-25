tc_TRS_QNAME_002() {
    local F="$1"
    run_scenario "name: in _index.md does not replace the directory name" "$F/TC-TRS-QNAME-002/name-override"
    assert_no_code "E005"
    assert_exit_zero
    assert_output_contains "'Pkg::Engine'"
    grep -qF "VS::Engine" <<<"$SCENARIO_OUTPUT" \
        && fail "label-derived qname VS::Engine reported" || pass "no label-derived qname VS::Engine"
}
