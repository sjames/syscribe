tc_TRS_CFLD_002() {
    local F="$1"; local M="$F/TC-TRS-CFLD-002/model"
    local out

    SCENARIO_NAME="exact match: custom.supplier=Bosch"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" ls --where custom.supplier=Bosch 2>/dev/null || true)
    printf '%s' "$out" | grep -q "Engine" && pass "matches Engine" || fail "Engine not matched"
    printf '%s' "$out" | grep -q "Gearbox" && fail "Gearbox wrongly matched" || pass "Gearbox excluded"

    SCENARIO_NAME="substring/regex: custom.costCenter=~PWT"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" ls --where custom.costCenter=~PWT 2>/dev/null || true)
    printf '%s' "$out" | grep -q "Engine" && printf '%s' "$out" | grep -q "Gearbox" \
        && pass "matches both PWT cost centers" || fail "substring match wrong"

    SCENARIO_NAME="list membership: custom.partNumbers~=A-1001"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" ls --where custom.partNumbers~=A-1001 2>/dev/null || true)
    printf '%s' "$out" | grep -q "Engine" && pass "matches list member" || fail "list membership failed"
    printf '%s' "$out" | grep -q "Gearbox" && fail "Gearbox wrongly matched" || pass "non-member excluded"

    SCENARIO_NAME="presence: custom.supplier"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" ls --where custom.supplier 2>/dev/null || true)
    printf '%s' "$out" | grep -q "Engine" && printf '%s' "$out" | grep -q "Gearbox" \
        && pass "presence matches both" || fail "presence match wrong"
    printf '%s' "$out" | grep -q "Wheel" && fail "Wheel (no custom fields) wrongly matched" \
        || pass "element without the field excluded"

    SCENARIO_NAME="unparseable predicate exits non-zero"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$M" ls --where 'bogus!!pred' >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -ne 0 ] && pass "bad predicate errors" || fail "bad predicate did not error"
}
