tc_TRS_MG_009() {
    local F="$1"; local B="$F/TC-TRS-MG-009"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a single SoI validates clean under the gate and is named in the grid report
    _scn "a single SoI validates clean and is identified in the grid report"
    out=$("$SYSCRIBE" -m "$B/ok" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -Eq 'MG06[0-9]'; then
        fail "single SoI unexpectedly produced an MG06x finding"
    else
        pass "no MG06x finding for a single SoI"
    fi
    out=$("$SYSCRIBE" -m "$B/ok" magicgrid 2>&1) || true
    if printf '%s' "$out" | grep -q 'System of interest:' && printf '%s' "$out" | grep -q 'Vehicle'; then
        pass "magicgrid report names Vehicle as the system of interest"
    else
        fail "magicgrid report did not name the system of interest"
    fi

    # 2. mg_soi on a non-part raises MG060
    _scn "mg_soi on a non-part raises MG060"
    out=$("$SYSCRIBE" -m "$B/mg060" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG060' \
        && pass "MG060 raised for mg_soi on a Requirement" \
        || fail "MG060 not raised for mg_soi on a Requirement"

    # 3. more than one SoI raises MG061
    _scn "more than one SoI raises MG061"
    out=$("$SYSCRIBE" -m "$B/mg061" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG061' \
        && pass "MG061 raised for two mg_soi parts" \
        || fail "MG061 not raised for two mg_soi parts"

    # 4. an SoI also marked external raises MG062
    _scn "an SoI also marked external raises MG062"
    out=$("$SYSCRIBE" -m "$B/mg062" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG062' \
        && pass "MG062 raised for mg_soi part also marked mg_external" \
        || fail "MG062 not raised for mg_soi part also marked mg_external"

    # 5. mg_soi is inert without the gate
    _scn "mg_soi is inert without the gate"
    out=$("$SYSCRIBE" -m "$B/mg061" validate 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -Eq 'MG06[0-9]'; then
        fail "an MG06x finding appeared without the gate"
    else
        pass "no MG06x finding without the gate"
    fi
}
