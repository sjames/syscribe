tc_TRS_MG_008() {
    local F="$1"; local B="$F/TC-TRS-MG-008"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a well-formed MoP validates clean under the gate and indexes its MoE
    _scn "a well-formed MoP validates clean and indexes its MoE"
    out=$("$SYSCRIBE" -m "$B/ok" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -Eq 'MG05[0-9]'; then
        fail "well-formed MoP unexpectedly produced an MG05x finding"
    else
        pass "no MG05x finding for a well-formed MoP"
    fi
    out=$("$SYSCRIBE" -m "$B/ok" show RangeMoE 2>&1) || true
    if printf '%s' "$out" | grep -q 'BatteryCapacityMoP'; then
        pass "MoE show lists the refining MoP BatteryCapacityMoP (mopRefinedBy)"
    else
        fail "MoE show did not report the refining MoP BatteryCapacityMoP (mopRefinedBy)"
    fi

    # 2. mg_mop on the wrong host raises MG050
    _scn "mg_mop on the wrong host raises MG050"
    out=$("$SYSCRIBE" -m "$B/mg050" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG050' \
        && pass "MG050 raised for mg_mop on a PartDef" \
        || fail "MG050 not raised for mg_mop on a PartDef"

    # 3. a missing or unresolved mg_mop_refines raises MG051
    _scn "a missing or unresolved mg_mop_refines raises MG051"
    out=$("$SYSCRIBE" -m "$B/mg051" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG051' \
        && pass "MG051 raised for unresolved mg_mop_refines" \
        || fail "MG051 not raised for unresolved mg_mop_refines"

    # 4. an mg_mop_refines target that is not an MoE raises MG052
    _scn "an mg_mop_refines target that is not an MoE raises MG052"
    out=$("$SYSCRIBE" -m "$B/mg052" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG052' \
        && pass "MG052 raised for mg_mop_refines pointing at a plain CalculationDef" \
        || fail "MG052 not raised for mg_mop_refines pointing at a plain CalculationDef"

    # 5. mg_mop fields are inert without the gate
    _scn "mg_mop fields are inert without the gate"
    out=$("$SYSCRIBE" -m "$B/mg050" validate 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -Eq 'MG05[0-9]'; then
        fail "an MG05x finding appeared without the gate"
    else
        pass "no MG05x finding without the gate"
    fi
}
