tc_TRS_MG_005() {
    local F="$1"; local B="$F/TC-TRS-MG-005"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. a bad mg_layer value raises MG040 under the gate
    _scn "a bad layer value raises MG040 under the gate"
    out=$("$SYSCRIBE" -m "$B/mg040" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG040' \
        && pass "MG040 raised for mg_layer: subsystem" \
        || fail "MG040 not raised for mg_layer: subsystem"

    # 2. an unrealised logical element raises MG041
    _scn "an unrealised logical element raises MG041"
    out=$("$SYSCRIBE" -m "$B/mg041" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG041' \
        && pass "MG041 raised for a logical part with no Allocation" \
        || fail "MG041 not raised for a logical part with no Allocation"

    # 3. cross-layer coupling (logical supertype is physical) raises MG042
    _scn "cross-layer coupling raises MG042"
    out=$("$SYSCRIBE" -m "$B/mg042" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG042' \
        && pass "MG042 raised for logical supertype on a physical part" \
        || fail "MG042 not raised for logical supertype on a physical part"

    # 4. routing through an Allocation clears MG042
    _scn "routing through an allocation clears MG042"
    out=$("$SYSCRIBE" -m "$B/mg042cleared" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG042' \
        && fail "MG042 still raised when the relation goes through an Allocation" \
        || pass "MG042 not raised when the relation goes through an Allocation"

    # 5. mg_layer is inert without the gate
    _scn "mg_layer is inert without the gate"
    out=$("$SYSCRIBE" -m "$B/mg040" validate 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -Eq 'MG04[0-9]'; then
        fail "an MG04x finding appeared without the gate"
    else
        pass "no MG04x finding without the gate"
    fi
}
