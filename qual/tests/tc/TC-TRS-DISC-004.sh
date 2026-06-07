tc_TRS_DISC_004() {
    local F="$1"; local B="$F/TC-TRS-DISC-001/pl"

    SCENARIO_NAME="list --feature filters to gated elements"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out ec
    out=$("$SYSCRIBE" -m "$B" list PartDef --feature Features::Engine::Electric 2>/dev/null) && ec=0 || ec=$?
    [ "${ec:-0}" -eq 0 ] && pass "list --feature exits 0" || fail "list --feature exit ${ec} (expected 0)"
    printf '%s' "$out" | grep -qF "Drivetrain::ElectricMotor" && pass "lists element gated on the feature" || fail "missing gated element ElectricMotor"
    # always-active element of the same type must NOT appear
    printf '%s' "$out" | grep -qF "Drivetrain::Frame" && fail "always-active Frame wrongly listed" || pass "always-active Frame excluded"

    SCENARIO_NAME="unknown feature errors"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B" list PartDef --feature Not::A::Feature >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "unknown feature exits non-zero" || fail "unknown --feature did not error"
}
