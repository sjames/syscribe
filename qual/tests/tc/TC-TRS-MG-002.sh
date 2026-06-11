tc_TRS_MG_002() {
    local F="$1"; local B="$F/TC-TRS-MG-002"
    local out rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. actors are inert without the gate (no actor finding on a dangling actor)
    _scn "actors are inert without the gate"
    out=$("$SYSCRIBE" -m "$B/mg010" validate 2>&1) && rc=0 || rc=$?
    if printf '%s' "$out" | grep -Eq 'MG01[0-9]'; then
        fail "an MG01x actor finding appeared without the gate"
    else
        pass "no MG01x finding without the gate"
    fi

    # 2. unresolved actor raises MG010 under the gate
    _scn "an unresolved actor raises MG010 under the gate"
    out=$("$SYSCRIBE" -m "$B/mg010" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG010' \
        && pass "MG010 raised for unresolved actor under the gate" \
        || fail "MG010 not raised for unresolved actor under the gate"

    # 3. actor that is not a part raises MG011
    _scn "an actor that is not a part raises MG011"
    out=$("$SYSCRIBE" -m "$B/mg011" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG011' \
        && pass "MG011 raised when actor names a Requirement" \
        || fail "MG011 not raised when actor names a Requirement"

    # 4. non-external actor raises MG012
    _scn "a non-external actor raises MG012"
    out=$("$SYSCRIBE" -m "$B/mg012" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG012' \
        && pass "MG012 raised when actor part is not mg_external" \
        || fail "MG012 not raised when actor part is not mg_external"

    # 5. use case with no actors raises MG013
    _scn "a use case with no actors raises MG013"
    out=$("$SYSCRIBE" -m "$B/mg013" validate --profile magicgrid 2>&1) && rc=0 || rc=$?
    printf '%s' "$out" | grep -q 'MG013' \
        && pass "MG013 raised for non-draft use case with empty actors" \
        || fail "MG013 not raised for non-draft use case with empty actors"

    # 6. actor participation is indexed (actorIn lists both use cases)
    _scn "actor participation is indexed"
    out=$("$SYSCRIBE" -m "$B/actorin" show Parts::Driver 2>&1) || true
    if printf '%s' "$out" | grep -q 'StopVehicle' && printf '%s' "$out" | grep -q 'StartVehicle'; then
        pass "actorIn on Driver lists both StopVehicle and StartVehicle"
    else
        fail "actorIn on Driver did not list both use cases"
    fi
}
