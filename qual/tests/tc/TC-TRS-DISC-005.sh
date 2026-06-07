tc_TRS_DISC_005() {
    local F="$1"; local B="$F/TC-TRS-DISC-001/pl"
    local EL="Drivetrain::ElectricMotor"   # gated on Features::Engine::Electric
    local ALWAYS="Drivetrain::Frame"        # no appliesWhen

    SCENARIO_NAME="active under a selecting config"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out ec
    out=$("$SYSCRIBE" -m "$B" why-active "$EL" --config CONF-PL-EV-001 2>/dev/null) && ec=0 || ec=$?
    [ "${ec:-0}" -eq 0 ] && pass "why-active exits 0 (selecting config)" || fail "exit ${ec} (expected 0)"
    printf '%s' "$out" | grep -qF "$EL" && pass "names the element" || fail "element qname missing"
    printf '%s' "$out" | grep -qF "Verdict: active" && pass "prints 'Verdict: active'" || fail "missing 'Verdict: active'"

    SCENARIO_NAME="inactive under a non-selecting config"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local o2 e2
    o2=$("$SYSCRIBE" -m "$B" why-active "$EL" --config CONF-PL-PETROL-001 2>/dev/null) && e2=0 || e2=$?
    [ "${e2:-0}" -eq 0 ] && pass "why-active exits 0 (non-selecting config)" || fail "exit ${e2} (expected 0)"
    printf '%s' "$o2" | grep -qF "Verdict: inactive" && pass "prints 'Verdict: inactive'" || fail "missing 'Verdict: inactive'"

    SCENARIO_NAME="always active for an ungated element"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local o3 e3
    o3=$("$SYSCRIBE" -m "$B" why-active "$ALWAYS" --config CONF-PL-PETROL-001 2>/dev/null) && e3=0 || e3=$?
    [ "${e3:-0}" -eq 0 ] && pass "why-active exits 0 (ungated element)" || fail "exit ${e3} (expected 0)"
    printf '%s' "$o3" | grep -qF "Verdict: always active" && pass "prints 'Verdict: always active'" || fail "missing 'Verdict: always active'"

    SCENARIO_NAME="--config is required and must resolve"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B" why-active "$EL" >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "missing --config exits non-zero" || fail "missing --config did not error"
    "$SYSCRIBE" -m "$B" why-active "$EL" --config CONF-DOES-NOT-EXIST >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "unresolved --config exits non-zero" || fail "unresolved --config did not error"
}
