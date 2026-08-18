tc_TRS_FM_005() {
    local F="$1"
    local FLAT="$F/TC-TRS-FM-005/flat"
    local NONAME="$F/TC-TRS-FM-005/noname"
    local DUPE="$F/TC-TRS-FM-005/dupe"
    local BADCONSTRAINT="$F/TC-TRS-FM-005/badconstraint"
    local WRONGTYPE="$F/TC-TRS-FM-005/wrongtype"

    SCENARIO_NAME="flat dotted featureTree: + crossTreeConstraints: validates clean and is deep-analysis sound"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out; out=$("$SYSCRIBE" -m "$FLAT" validate 2>/dev/null || true)
    SCENARIO_OUTPUT="$out"
    printf '%s' "$out" | grep -qE '\| E[0-9]+ \|' \
        && fail "no E-level finding expected on flat featureTree model" \
        || pass "no E-level finding on flat featureTree model"

    local deep; deep=$("$SYSCRIBE" -m "$FLAT" feature-check --deep 2>/dev/null || true)
    SCENARIO_OUTPUT="$deep"
    assert_no_code "E223"
    assert_no_code "E225"
    local core; core=$(printf '%s' "$deep" | grep -F "core features:" | head -1 || true)
    printf '%s' "$core" | grep -qF "Features::Platform" \
        && pass "Features::Platform on core features line" || fail "Features::Platform not on core features line"

    local fc; fc=$("$SYSCRIBE" -m "$FLAT" feature-check 2>/dev/null || true)
    SCENARIO_OUTPUT="$fc"
    assert_no_code "E213"

    SCENARIO_NAME="featureTree entry with no name is dropped and flagged E231"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$NONAME" validate 2>/dev/null || true)
    assert_has_code "E231"

    SCENARIO_NAME="two featureTree entries colliding on qname flagged E232"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$DUPE" validate 2>/dev/null || true)
    assert_has_code "E232"

    SCENARIO_NAME="crossTreeConstraints feature not resolving within the sheet flagged E233"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BADCONSTRAINT" validate 2>/dev/null || true)
    assert_has_code "E233"

    SCENARIO_NAME="featureTree on a non-FeatureModel type flagged W048"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$WRONGTYPE" validate 2>/dev/null || true)
    assert_has_code "W048"
}
