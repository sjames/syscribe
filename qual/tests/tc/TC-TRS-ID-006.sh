tc_TRS_ID_006() {
    local F="$1"; local B="$F/TC-TRS-ID-006"

    # ── id-form: feature referenced by FEAT id in appliesWhen + Configuration ──
    SCENARIO_NAME="FEAT-id appliesWhen/Configuration validate without E209"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/id-form" validate 2>/dev/null || true)
    assert_no_code "E209"
    assert_no_code "W042"

    SCENARIO_NAME="FEAT-id requirement is active in the selecting configuration"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local on; on=$("$SYSCRIBE" -m "$B/id-form" list Requirement --config CONF-ID6-ABS-001 2>/dev/null || true)
    printf '%s' "$on" | grep -qF "Reqs::AbsReq" \
        && pass "FEAT-id req active when feature selected" || fail "FEAT-id req not active when selected"

    SCENARIO_NAME="FEAT-id requirement is inactive in the deselecting configuration"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local off; off=$("$SYSCRIBE" -m "$B/id-form" list Requirement --config CONF-ID6-NOABS-001 2>/dev/null || true)
    printf '%s' "$off" | grep -qF "Reqs::AbsReq" \
        && fail "FEAT-id req wrongly active when deselected" || pass "FEAT-id req inactive when deselected"

    SCENARIO_NAME="why-active reports the same verdict for the FEAT-id form"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local wa_on wa_off
    wa_on=$("$SYSCRIBE" -m "$B/id-form" why-active REQ-ID6-ABS-001 --config CONF-ID6-ABS-001 2>/dev/null || true)
    wa_off=$("$SYSCRIBE" -m "$B/id-form" why-active REQ-ID6-ABS-001 --config CONF-ID6-NOABS-001 2>/dev/null || true)
    printf '%s' "$wa_on"  | grep -qiF "Verdict: active"   && pass "why-active = active when selected"   || fail "why-active not active when selected"
    printf '%s' "$wa_off" | grep -qiF "Verdict: inactive" && pass "why-active = inactive when deselected" || fail "why-active not inactive when deselected"

    # ── qname-form: same feature referenced by qualified name (equivalence) ────
    SCENARIO_NAME="qname-form yields identical activation to the FEAT-id form"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local qon qoff
    qon=$("$SYSCRIBE" -m "$B/qname-form" list Requirement --config CONF-ID6-ABS-001 2>/dev/null || true)
    qoff=$("$SYSCRIBE" -m "$B/qname-form" list Requirement --config CONF-ID6-NOABS-001 2>/dev/null || true)
    printf '%s' "$qon"  | grep -qF "Reqs::AbsReq" && pass "qname-form req active when selected"  || fail "qname-form req not active when selected"
    printf '%s' "$qoff" | grep -qF "Reqs::AbsReq" && fail "qname-form req wrongly active when deselected" || pass "qname-form req inactive when deselected"

    # ── negatives ─────────────────────────────────────────────────────────────
    SCENARIO_NAME="a hyphenated NAME reference still raises E209 (no GH #42 regression)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/neg-hyphen-name" validate 2>/dev/null || true)
    assert_has_code "E209"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E209 |" | grep -qF "'-'" \
        && pass "E209 rejects the hyphen in the name reference" || fail "E209 does not flag the hyphen"

    SCENARIO_NAME="a malformed FEAT id raises E006"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/neg-bad-id" validate 2>/dev/null || true)
    assert_has_code "E006"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E006 |" | grep -qF "FEAT pattern" \
        && pass "E006 names the FEAT pattern" || fail "E006 does not mention FEAT pattern"

    SCENARIO_NAME="two FeatureDefs sharing a FEAT id raise E101"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/neg-dup-id" validate 2>/dev/null || true)
    assert_has_code "E101"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E101 |" | grep -qF "FEAT-ABS-001" \
        && pass "E101 names the duplicated FEAT id" || fail "E101 does not name FEAT-ABS-001"

    SCENARIO_NAME="a stable-id-shaped reference resolving to nothing raises E209"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/neg-unresolved" validate 2>/dev/null || true)
    assert_has_code "E209"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E209 |" | grep -qF "FEAT-NOPE-001" \
        && pass "E209 names the unresolved FEAT id" || fail "E209 does not name FEAT-NOPE-001"
}
