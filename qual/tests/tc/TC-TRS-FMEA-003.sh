tc_TRS_FMEA_003() {
    local F="$1"; local M="$F/TC-TRS-FMEA-003/model"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" validate 2>/dev/null || true)

    SCENARIO_NAME="fmeaRef to existing FMEAEntry does not raise W926"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W926 |" | grep -qF "FTE-KERN-001" \
        && fail "W926 wrongly fired for FTE-KERN-001 which has a valid fmeaRef" \
        || pass "no W926 for FTE-KERN-001 (fmeaRef resolves)"

    SCENARIO_NAME="fmeaRef to non-existent element raises W926"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W926 |" | grep -qF "FM-NONEXIST-001" \
        && pass "W926 raised for FM-NONEXIST-001 (unresolvable fmeaRef)" \
        || fail "W926 not raised for FM-NONEXIST-001"

    SCENARIO_NAME="ftaRef to existing FaultTreeEvent does not raise W927"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W927 |" | grep -qF "FTE-KERN-001" \
        && fail "W927 wrongly fired for FM-KERN-001 which has a valid ftaRef" \
        || pass "no W927 for FM-KERN-001 (ftaRef resolves)"

    SCENARIO_NAME="ftaRef to non-existent element raises W927"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W927 |" | grep -qF "FTE-NONEXIST-001" \
        && pass "W927 raised for FTE-NONEXIST-001 (unresolvable ftaRef)" \
        || fail "W927 not raised for FTE-NONEXIST-001"

    SCENARIO_NAME="refs FM-KERN-001 lists FTE-KERN-001 as referencing via fmeaRef"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    refs=$("$SYSCRIBE" -m "$M" refs FM-KERN-001 2>/dev/null || true)
    printf '%s' "$refs" | grep -qF "FTE-KERN-001" \
        && pass "refs FM-KERN-001 lists FTE-KERN-001" \
        || fail "refs FM-KERN-001 does not list FTE-KERN-001"
    printf '%s' "$refs" | grep "FTE-KERN-001" | grep -qF "fmeaRef" \
        && pass "refs relationship is fmeaRef" \
        || fail "refs relationship not shown as fmeaRef"
}
