tc_TRS_OUT_009() {
    local F="$1"
    local RES="$F/TC-TRS-OUT-009/results"
    local NORES="$F/TC-TRS-OUT-009/noresults"

    # ── Scenario 1: matrix distinguishes passing from covered-not-passing ──────
    SCENARIO_NAME="matrix shows ✓ for passing and ▣ for covered-not-passing"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$RES" matrix 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "REQ-OUT9-PASS-001 | ✓" \
        && pass "passing requirement shows ✓" || fail "passing requirement missing ✓"
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "REQ-OUT9-FAIL-001 | ▣" \
        && pass "failing requirement shows ▣" || fail "failing requirement missing ▣"
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "covered, not passing" \
        && pass "legend mentions covered, not passing" || fail "legend missing covered-not-passing"

    # ── Scenario 2: trace annotates verifying TestCases with the verdict ───────
    SCENARIO_NAME="trace annotates verifying TestCases with ingested verdict"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$RES" trace REQ-OUT9-PASS-001 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "TC-OUT9-PASS-001 [pass]" \
        && pass "passing TestCase annotated [pass]" || fail "passing TestCase missing [pass]"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$RES" trace REQ-OUT9-FAIL-001 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "TC-OUT9-FAIL-001 [fail]" \
        && pass "failing TestCase annotated [fail]" || fail "failing TestCase missing [fail]"

    # ── Scenario 3: --linked-only reverts matrix to the plain covered glyph ────
    SCENARIO_NAME="matrix --linked-only reverts to ✓ with no ▣"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$RES" matrix --linked-only 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "▣" \
        && fail "▣ present under --linked-only" || pass "no ▣ under --linked-only"
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "REQ-OUT9-FAIL-001 | ✓" \
        && pass "covered cell shows ✓ under --linked-only" || fail "covered cell not ✓ under --linked-only"

    # ── Scenario 4: trace --linked-only reverts to no verdict annotation ──────
    SCENARIO_NAME="trace --linked-only drops verdict annotation"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$RES" trace REQ-OUT9-FAIL-001 --linked-only 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    printf '%s' "$SCENARIO_OUTPUT" | grep -qE "\[(pass|fail|unknown)\]" \
        && fail "verdict annotation present under --linked-only" || pass "no verdict annotation under --linked-only"

    # ── Scenario 5: a model with no sidecar degrades gracefully ───────────────
    SCENARIO_NAME="no sidecar → no ▣ glyph and no verdict annotations"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$NORES" matrix 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "▣" \
        && fail "▣ present without a sidecar" || pass "no ▣ without a sidecar"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$NORES" trace REQ-OUT9-FAIL-001 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    printf '%s' "$SCENARIO_OUTPUT" | grep -qE "\[(pass|fail|unknown)\]" \
        && fail "verdict annotation present without a sidecar" || pass "no verdict annotation without a sidecar"
}
