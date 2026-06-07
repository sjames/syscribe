tc_TRS_DISC_002() {
    local F="$1"; local B="$F/TC-TRS-DISC-001/pl"

    SCENARIO_NAME="feature card for a gated XOR child"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out ec
    out=$("$SYSCRIBE" -m "$B" feature Features::Engine::Electric 2>/dev/null) && ec=0 || ec=$?
    [ "${ec:-0}" -eq 0 ] && pass "feature exits 0" || fail "feature exit ${ec} (expected 0)"
    printf '%s' "$out" | grep -qF "# Feature:" && pass "shows '# Feature:' card header" || fail "missing '# Feature:' header"
    printf '%s' "$out" | grep -qF "Features::Engine::Electric" && pass "shows feature qname" || fail "missing feature qname"
    # Gates section names the gated elements
    printf '%s' "$out" | grep -qF "Gates" && pass "shows 'Gates' section" || fail "missing 'Gates' section"
    printf '%s' "$out" | grep -qF "Drivetrain::ElectricMotor" && pass "Gates lists ElectricMotor" || fail "Gates missing ElectricMotor"
    # Selecting configurations
    printf '%s' "$out" | grep -qF "Selected in" && pass "shows 'Selected in'" || fail "missing 'Selected in'"
    printf '%s' "$out" | grep -qF "CONF-PL-EV-001" && pass "names selecting config CONF-PL-EV-001" || fail "missing selecting config id"

    SCENARIO_NAME="feature card shows parameters"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local p pec
    p=$("$SYSCRIBE" -m "$B" feature Features::Battery 2>/dev/null) && pec=0 || pec=$?
    [ "${pec:-0}" -eq 0 ] && pass "feature exits 0 for parameterised feature" || fail "exit ${pec} (expected 0)"
    printf '%s' "$p" | grep -qF "capacityKwh" && pass "param name appears on card" || fail "missing parameter name"

    SCENARIO_NAME="unknown feature errors"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B" feature Not::A::Feature >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "unknown feature exits non-zero" || fail "unknown feature did not error"
}
