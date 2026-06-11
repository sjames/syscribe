tc_TRS_MG_004() {
    local F="$1"; local B="$F/TC-TRS-MG-004"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a well-formed MoE validates clean under the gate (no MG03x)
    _scn "a well-formed MoE validates clean under the gate"
    out=$("$SYSCRIBE" -m "$B/ok" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -Eq 'MG03[0-9]'; then
        fail "well-formed MoE unexpectedly produced an MG03x finding"
    else
        pass "no MG03x finding for a well-formed MoE"
    fi

    # 2. mg_moe on the wrong host raises MG030
    _scn "mg_moe on the wrong host raises MG030"
    out=$("$SYSCRIBE" -m "$B/mg030" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG030' \
        && pass "MG030 raised for mg_moe on a PartDef" \
        || fail "MG030 not raised for mg_moe on a PartDef"

    # 3. a missing or unresolved measures raises MG031
    _scn "a missing or unresolved measures raises MG031"
    out=$("$SYSCRIBE" -m "$B/mg031" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG031' \
        && pass "MG031 raised for unresolved mg_moe_measures" \
        || fail "MG031 not raised for unresolved mg_moe_measures"

    # 4. a bad direction raises MG032
    _scn "a bad direction raises MG032"
    out=$("$SYSCRIBE" -m "$B/mg032" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG032' \
        && pass "MG032 raised for mg_moe_direction: bigger" \
        || fail "MG032 not raised for mg_moe_direction: bigger"

    # 5. inconsistent bounds raise MG033
    _scn "inconsistent bounds raise MG033"
    out=$("$SYSCRIBE" -m "$B/mg033" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG033' \
        && pass "MG033 raised for maximize objective < threshold" \
        || fail "MG033 not raised for maximize objective < threshold"

    # 6. mg_moe fields are inert without the gate
    _scn "mg_moe fields are inert without the gate"
    out=$("$SYSCRIBE" -m "$B/mg030" validate 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -Eq 'MG03[0-9]'; then
        fail "an MG03x finding appeared without the gate"
    else
        pass "no MG03x finding without the gate"
    fi
}
