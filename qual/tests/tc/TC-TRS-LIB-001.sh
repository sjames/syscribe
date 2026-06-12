tc_TRS_LIB_001() {
    local F="$1"; local G="$F/TC-TRS-LIB-001/good"; local T="$F/TC-TRS-LIB-001/typo"

    SCENARIO_NAME="recognised built-in members resolve cleanly (no W404, no W043)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" validate 2>/dev/null || true)
    assert_no_code "W404"
    assert_no_code "W043"

    SCENARIO_NAME="an import-only package (SI) reference is not flagged W043"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W043 |" | grep -qF "SI::" \
        && fail "W043 wrongly flagged an import-only SI reference" || pass "SI reference left lenient"

    SCENARIO_NAME="an unknown member of a known built-in package raises W043"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$T" validate 2>/dev/null || true)
    assert_has_code "W043"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W043 |" | grep -qF "ScalarValues::Flota" \
        && pass "W043 names the bad ScalarValues member" || fail "W043 does not name ScalarValues::Flota"

    SCENARIO_NAME="W043 lists the package's known members"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W043 |" | grep -qF "known members: Integer, Real, Natural, Boolean, String" \
        && pass "W043 lists ScalarValues members" || fail "W043 does not list known members"

    SCENARIO_NAME="the check covers Base and multiple contexts (supertype, typedBy, returnType, parameter type)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    { printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W043 |" | grep -qF "Base::Nope" \
        && printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W043 |" | grep -qF "ScalarValues::Flta" \
        && printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W043 |" | grep -qF "Base::Booleen"; } \
        && pass "W043 fires across supertype/typedBy/returnType/parameter type" || fail "W043 missed a context"
}
