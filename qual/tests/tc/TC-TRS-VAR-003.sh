tc_TRS_VAR_003() {
    local F="$1"

    # Scenario: an unresolved operand inside an expression is E209.
    run_scenario "unresolved operand in expression is E209" "$F/TC-TRS-VAR-003/unresolved-operand"
    assert_has_code "E209"

    # Scenarios: evaluate and/or/not/parens + bare-QName against four configurations,
    # observed through the matrix. A requirement with no test is 'gap' where active,
    # 'na' where its appliesWhen is not satisfied.
    local M="$F/TC-TRS-VAR-003/truth-table"
    local J; J=$("$SYSCRIBE" -m "$M" matrix --json 2>/dev/null || true)

    # cell <id> <config> -> prints the cell state (covered|gap|na)
    cell() { printf '%s' "$J" | jq -r --arg id "$1" --arg c "$2" \
        '.rows[] | select(.id==$id) | .cells[$c]' 2>/dev/null || true; }
    expect() { # id, config, want, label
        local got; got=$(cell "$1" "$2")
        [ "$got" = "$3" ] && pass "$4: $1 @ $2 = $3" || fail "$4: $1 @ $2 = '$got' (expected $3)"
    }

    SCENARIO_NAME="AND is active only where both selected"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    expect REQ-V3-AND-001 CONF-TT-AB-001   gap AND
    expect REQ-V3-AND-001 CONF-TT-ANB-001  na  AND
    expect REQ-V3-AND-001 CONF-TT-NAB-001  na  AND

    SCENARIO_NAME="OR is N/A only where neither selected"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    expect REQ-V3-OR-001 CONF-TT-AB-001   gap OR
    expect REQ-V3-OR-001 CONF-TT-NAB-001  gap OR
    expect REQ-V3-OR-001 CONF-TT-NANB-001 na  OR

    SCENARIO_NAME="NOT is active exactly where deselected"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    expect REQ-V3-NOT-001 CONF-TT-AB-001   na  NOT
    expect REQ-V3-NOT-001 CONF-TT-NAB-001  gap NOT

    SCENARIO_NAME="parentheses: (A or B) and not A"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    expect REQ-V3-PAREN-001 CONF-TT-NAB-001  gap PAREN
    expect REQ-V3-PAREN-001 CONF-TT-AB-001   na  PAREN
    expect REQ-V3-PAREN-001 CONF-TT-NANB-001 na  PAREN

    SCENARIO_NAME="bare QName remains back-compatible"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    expect REQ-V3-BARE-001 CONF-TT-AB-001  gap BARE
    expect REQ-V3-BARE-001 CONF-TT-NAB-001 na  BARE
}
