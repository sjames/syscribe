tc_TRS_TYPE_017() {
    local F="$1"

    SCENARIO_NAME="valid Asset validates without E861/E862/E863/W810"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-TYPE-017/model" validate 2>/dev/null || true)
    printf '%s' "$out" | grep -qE "E861|E862|E863|E864" \
        && fail "error codes fired for valid Asset model" \
        || pass "no E861/E862/E863/E864 for valid Asset"
    printf '%s' "$out" | grep -qF "W810" \
        && fail "W810 fired when Asset IS referenced" \
        || pass "no W810 when Asset is referenced by DamageScenario"

    SCENARIO_NAME="Asset bad id pattern triggers E862"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$F/TC-TRS-TYPE-017/model-errors" validate 2>/dev/null || true)
    printf '%s' "$out" | grep -qF "E862" \
        && pass "E862 emitted for bad ASSET id pattern" \
        || fail "E862 not emitted for bad ASSET id"

    SCENARIO_NAME="Asset bad cybersecurityProperties triggers E863"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$out" | grep -qF "E863" \
        && pass "E863 emitted for invalid cybersecurityProperties" \
        || fail "E863 not emitted for invalid cybersecurityProperties"

    SCENARIO_NAME="Asset with no DamageScenario reference triggers W810"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    # ASSET-TST-002 in model-errors has bad cp but also is not referenced by any DS
    printf '%s' "$out" | grep -qF "W810" \
        && pass "W810 emitted for unreferenced Asset" \
        || fail "W810 not emitted for unreferenced Asset"

    SCENARIO_NAME="list Asset shows asset elements"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    list=$("$SYSCRIBE" -m "$F/TC-TRS-TYPE-017/model" list Asset 2>/dev/null || true)
    printf '%s' "$list" | grep -qF "ASSET-TST-001" \
        && pass "list Asset includes ASSET-TST-001" \
        || fail "list Asset missing ASSET-TST-001"
}
