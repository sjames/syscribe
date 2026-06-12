tc_TRS_FMEA_002() {
    local F="$1"; local M="$F/TC-TRS-FMEA-002/model"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" validate 2>/dev/null || true)

    SCENARIO_NAME="fmeaSeverity-keyed entry with implicit RPN triggers W903"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W903 |" | grep -qF "729" \
        && pass "W903 fires with computed RPN 729" || fail "W903 not fired (fmeaSeverity RPN auto-compute broken)"

    SCENARIO_NAME="deprecated severity: alias is accepted without diagnostic"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E922 |" | grep -qF "severity" \
        && fail "E922 wrongly fired on deprecated severity: alias" || pass "deprecated severity: accepted without E922"

    SCENARIO_NAME="unknown key failureEffect raises E922"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E922 |" | grep -qF "failureEffect" \
        && pass "E922 raised for unknown key failureEffect" || fail "E922 not raised for failureEffect"

    SCENARIO_NAME="template FMEASheet emits fmeaSeverity not severity"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    tpl=$("$SYSCRIBE" template FMEASheet 2>/dev/null || true)
    printf '%s' "$tpl" | grep -qE "^\s+fmeaSeverity:" \
        && pass "template emits fmeaSeverity:" || fail "template missing fmeaSeverity:"
    printf '%s' "$tpl" | grep -qE "^\s+severity:" \
        && fail "template still emits deprecated severity:" || pass "template has no bare severity: line"
}
