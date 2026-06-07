tc_TRS_DISC_001() {
    local F="$1"; local B="$F/TC-TRS-DISC-001/pl"
    local NOFM="$F/TC-TRS-FM-001/no-fm"

    SCENARIO_NAME="features prints the feature-model overview"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out ec
    out=$("$SYSCRIBE" -m "$B" features 2>/dev/null) && ec=0 || ec=$?
    [ "${ec:-0}" -eq 0 ] && pass "features exits 0" || fail "features exit ${ec} (expected 0)"
    printf '%s' "$out" | grep -qF "# Feature Model" && pass "header present" || fail "missing '# Feature Model' header"
    # every feature qualified name appears
    for q in "Features::Core" "Features::Engine" "Features::Engine::Petrol" \
             "Features::Engine::Electric" "Features::Battery" "Features::Sunroof"; do
        printf '%s' "$out" | grep -qF "$q" && pass "lists $q" || fail "missing feature $q"
    done
    # groupKind of the XOR group
    printf '%s' "$out" | grep -qF "alternative" && pass "shows groupKind alternative" || fail "missing groupKind 'alternative'"
    # parameter name of the parameterised feature
    printf '%s' "$out" | grep -qF "capacityKwh" && pass "shows parameter name" || fail "missing parameter 'capacityKwh'"
    # per-feature selection rollup substring
    printf '%s' "$out" | grep -qF "selected in " && pass "shows 'selected in ' rollup" || fail "missing 'selected in ' rollup"

    SCENARIO_NAME="features --json emits a JSON document"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local j jec first
    j=$("$SYSCRIBE" -m "$B" features --json 2>/dev/null) && jec=0 || jec=$?
    [ "${jec:-0}" -eq 0 ] && pass "features --json exits 0" || fail "features --json exit ${jec} (expected 0)"
    first=$(printf '%s' "$j" | tr -d '[:space:]' | cut -c1)
    { [ "$first" = "{" ] || [ "$first" = "[" ]; } && pass "json starts with { or [" || fail "json does not start with { or [ (got '${first}')"

    SCENARIO_NAME="dormant on a model with no feature model"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local d dec
    d=$("$SYSCRIBE" -m "$NOFM" features 2>/dev/null) && dec=0 || dec=$?
    [ "${dec:-0}" -eq 0 ] && pass "features exits 0 with no feature model" || fail "features exit ${dec} (expected 0)"
    printf '%s' "$d" | grep -qiF "no feature model" && pass "prints 'no feature model' notice" || fail "missing 'no feature model' notice"
}
