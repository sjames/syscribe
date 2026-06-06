tc_TRS_VAR_004() {
    local F="$1"; local M="$F/TC-TRS-VAR-004/model"

    # matrix --json contract (pins the implementation):
    #   { "schemaVersion": "1.0",
    #     "columns": ["CONF-MPS2-WDT-001","CONF-M0-BASE-001"],
    #     "rows": [ {"id":"REQ-...","cells":{"CONF-...":"covered|gap|na"}} ] }
    local J; J=$("$SYSCRIBE" -m "$M" matrix --json 2>/dev/null || true)

    cell() { printf '%s' "$J" | jq -r --arg id "$1" --arg c "$2" \
        '.rows[] | select(.id==$id) | .cells[$c]' 2>/dev/null || true; }
    expect() { local got; got=$(cell "$1" "$2")
        [ "$got" = "$3" ] && pass "$1 @ $2 = $3" || fail "$1 @ $2 = '$got' (expected $3)"; }

    # Scenario: columns are exactly the model's Configuration elements + schemaVersion.
    SCENARIO_NAME="columns are the model Configuration elements"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local sv; sv=$(printf '%s' "$J" | jq -r '.schemaVersion' 2>/dev/null || true)
    [ "$sv" = "1.0" ] && pass "schemaVersion is 1.0" || fail "schemaVersion='$sv' (expected 1.0)"
    local cols; cols=$(printf '%s' "$J" | jq -r '.columns | sort | join(",")' 2>/dev/null || true)
    [ "$cols" = "CONF-M0-BASE-001,CONF-MPS2-WDT-001" ] \
        && pass "columns are the two configuration ids" || fail "columns='$cols'"

    # Scenario: unconditional requirement covered everywhere.
    SCENARIO_NAME="unconditional requirement covered in every configuration"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    expect REQ-V4-CORE-001 CONF-MPS2-WDT-001 covered
    expect REQ-V4-CORE-001 CONF-M0-BASE-001  covered

    # Scenario: conditioned + covered requirement: covered where active, N/A elsewhere.
    SCENARIO_NAME="conditioned requirement is N/A where feature deselected"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    expect REQ-V4-WDT-001 CONF-MPS2-WDT-001 covered
    expect REQ-V4-WDT-001 CONF-M0-BASE-001  na

    # Scenario: active requirement with no in-config test is a gap.
    SCENARIO_NAME="active requirement with no covering test is a gap"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    expect REQ-V4-WDT-002 CONF-MPS2-WDT-001 gap
    expect REQ-V4-WDT-002 CONF-M0-BASE-001  na

    # Scenario: text mode is a smoke check (exits 0, shows a column + a glyph).
    SCENARIO_NAME="text matrix renders configurations and cells"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local txt; txt=$("$SYSCRIBE" -m "$M" matrix 2>/dev/null) && local ex=0 || local ex=$?
    [ "${ex:-0}" -eq 0 ] && pass "matrix exits 0" || fail "matrix exit ${ex} (expected 0)"
    printf '%s' "$txt" | grep -qF "CONF-MPS2-WDT-001" \
        && pass "text matrix shows a configuration column" || fail "text matrix missing configuration column"
}
