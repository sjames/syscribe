tc_TRS_FM_006() {
    local F="$1"
    local DERIVE="$F/TC-TRS-FM-006/derive"
    local BADDERIVE="$F/TC-TRS-FM-006/badderive"
    local DUPDERIVE="$F/TC-TRS-FM-006/dupderive"
    local PERFILE="$F/TC-TRS-FM-006/perfile"

    SCENARIO_NAME="featureTree: entries with no id: derive one from name; explicit id: still wins"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out; out=$("$SYSCRIBE" -m "$DERIVE" validate 2>/dev/null || true)
    SCENARIO_OUTPUT="$out"
    printf '%s' "$out" | grep -qE '\| E[0-9]+ \|' \
        && fail "no E-level finding expected on the derive fixture" \
        || pass "no E-level finding on the derive fixture"

    local wdt_id; wdt_id=$("$SYSCRIBE" -m "$DERIVE" show Features::Wdt 2>/dev/null | grep -F '**id**' || true)
    printf '%s' "$wdt_id" | grep -qF "FEAT-WDT" \
        && pass "Wdt derives FEAT-WDT" || fail "Wdt id not FEAT-WDT: $wdt_id"

    local cortexm_id; cortexm_id=$("$SYSCRIBE" -m "$DERIVE" show Features::Platform::CortexM 2>/dev/null | grep -F '**id**' || true)
    printf '%s' "$cortexm_id" | grep -qF "FEAT-PLATFORM-CORTEXM" \
        && pass "Platform.CortexM derives FEAT-PLATFORM-CORTEXM" || fail "CortexM id wrong: $cortexm_id"

    local custom_id; custom_id=$("$SYSCRIBE" -m "$DERIVE" show Features::Custom 2>/dev/null | grep -F '**id**' || true)
    printf '%s' "$custom_id" | grep -qF "FEAT-CUSTOM-001" \
        && pass "explicit id: FEAT-CUSTOM-001 overrides derivation" || fail "Custom id wrong: $custom_id"

    SCENARIO_NAME="grammar-invalid derived id surfaces the existing E006"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$BADDERIVE" validate 2>/dev/null || true)
    assert_has_code "E006"

    SCENARIO_NAME="two entries deriving to the same id collide as E101"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$DUPDERIVE" validate 2>/dev/null || true)
    assert_has_code "E101"

    SCENARIO_NAME="a plain per-file FeatureDef with no id: is unaffected — still E201"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$PERFILE" validate 2>/dev/null || true)
    assert_has_code "E201"
}
