tc_TRS_MG_011() {
    local F="$1"; local B="$F/TC-TRS-MG-011"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a marked Configuration without featureModel validates with no E201
    _scn "a marked Configuration without featureModel validates with no E201"
    out=$("$SYSCRIBE" -m "$B/ok" validate 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -q 'E201'; then
        fail "E201 wrongly raised for an mg_variant Configuration with no featureModel"
    else
        pass "no E201 for an mg_variant Configuration with no featureModel"
    fi

    # 2. an unmarked Configuration without featureModel still raises E201
    _scn "an unmarked Configuration without featureModel still raises E201"
    out=$("$SYSCRIBE" -m "$B/unmarked" validate 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'E201' \
        && pass "E201 raised for an unmarked Configuration with no featureModel" \
        || fail "E201 not raised for an unmarked Configuration with no featureModel"

    # 3. trade-study scores a parametric variant as a column
    _scn "trade-study scores a parametric variant from its parameterBindings"
    out=$("$SYSCRIBE" -m "$B/ok" trade-study 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -q 'Variant A'; then
        pass "trade-study shows the parametric variant as a scored column"
    else
        fail "trade-study did not show the parametric variant column"
    fi

    # 4. validate --config on a parametric variant projects the identity without panicking
    _scn "validate --config on a parametric variant projects the identity"
    out=$("$SYSCRIBE" -m "$B/ok" validate --config CONF-MG-VARIANT-001 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -qiE 'panic|thread .main. panicked'; then
        fail "validate --config panicked on a parametric variant"
    elif [ "$rc" -eq 0 ]; then
        pass "validate --config returned rc 0 with no panic on a parametric variant"
    else
        fail "validate --config returned rc $rc on a parametric variant"
    fi

    # 5. mg_variant on a non-Configuration raises MG070 under the gate
    _scn "mg_variant on a non-Configuration raises MG070 under the gate"
    out=$("$SYSCRIBE" -m "$B/mg070" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG070' \
        && pass "MG070 raised for mg_variant on a PartDef" \
        || fail "MG070 not raised for mg_variant on a PartDef"
}
