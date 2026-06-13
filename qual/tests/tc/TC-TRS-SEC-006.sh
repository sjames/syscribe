tc_TRS_SEC_006() {
    local F="$1"; local M="$F/TC-TRS-SEC-006/model"

    SCENARIO_NAME="derivedFromCybersecurityGoal resolves cleanly"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" validate 2>/dev/null || true)
    printf '%s' "$out" | grep -qF "E831" \
        && fail "E831 fired for valid derivedFromCybersecurityGoal" \
        || pass "no E831 for valid derivedFromCybersecurityGoal"

    SCENARIO_NAME="legacy derivedFromSecurityGoal alias also resolves"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    # REQ-DR-002 uses the old key name; if it parses correctly E831 should not fire for it
    printf '%s' "$out" | grep -qE "REQ-DR-002.*E831|E831.*REQ-DR-002" \
        && fail "E831 fired for legacy alias derivedFromSecurityGoal" \
        || pass "legacy alias derivedFromSecurityGoal works without E831"
}
