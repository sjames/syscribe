tc_TRS_MG_001() {
    local F="$1"; local B="$F/TC-TRS-MG-001"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. refines parses, both targets resolve, refinedBy back-links the requirement
    _scn "a refines link parses and back-links the requirement"
    out=$("$SYSCRIBE" -m "$B/ok" validate 2>&1) && rc=0 || rc=$?
    # clean model: no E316, and the refines targets resolved (no unresolved-ref error)
    if printf '%s' "$out" | grep -q 'E316'; then
        fail "ok model unexpectedly raised E316"
    else
        pass "ok model: no E316 (both refines targets resolve)"
    fi
    # refinedBy reverse index surfaced on the requirement's show output
    out=$("$SYSCRIBE" -m "$B/ok" show REQ-MG-BRAKE-001 2>&1) || true
    printf '%s' "$out" | grep -q 'StopVehicle' \
        && pass "refinedBy surfaces StopVehicle on REQ-MG-BRAKE-001" \
        || fail "refinedBy did not surface StopVehicle on REQ-MG-BRAKE-001"
    out=$("$SYSCRIBE" -m "$B/ok" show REQ-MG-STEER-001 2>&1) || true
    printf '%s' "$out" | grep -q 'StopVehicle' \
        && pass "refinedBy surfaces StopVehicle on REQ-MG-STEER-001 (resolved by qname)" \
        || fail "refinedBy did not surface StopVehicle on REQ-MG-STEER-001"

    # 2. unresolved refines target raises E316
    _scn "an unresolved refines target raises E316"
    out=$("$SYSCRIBE" -m "$B/e316unresolved" validate 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'E316' \
        && pass "E316 raised for unresolved refines target" \
        || fail "E316 not raised for unresolved refines target"

    # 3. refines target that is not a requirement raises E316
    _scn "a refines target that is not a requirement raises E316"
    out=$("$SYSCRIBE" -m "$B/e316nonreq" validate 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'E316' \
        && pass "E316 raised when refines names a PartDef" \
        || fail "E316 not raised when refines names a PartDef"

    # 4. non-draft use case with no refines warns W307; default exit unaffected (warning)
    _scn "a non-draft use case with no refines warns W307"
    out=$("$SYSCRIBE" -m "$B/w307" validate 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -q 'W307' && [ "$rc" -eq 0 ]; then
        pass "W307 raised; plain validate exit code 0 (warning)"
    else
        fail "W307/exit-0 expectation not met (rc=$rc)"
    fi

    # 5. draft use case with no refines is suppressed (no W307)
    _scn "a draft use case with no refines is suppressed"
    out=$("$SYSCRIBE" -m "$B/draft" validate 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'W307' \
        && fail "W307 not suppressed on draft use case" \
        || pass "no W307 on draft use case"

    # 6. magicgrid profile promotes W307 to a gate failure (non-zero exit)
    _scn "the magicgrid profile promotes W307 to a gate failure"
    out=$("$SYSCRIBE" -m "$B/w307" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    [ "$rc" -ne 0 ] \
        && pass "validate --profile magicgrid exits non-zero (W307 promoted)" \
        || fail "magicgrid profile did not promote W307 (rc=$rc)"
}
