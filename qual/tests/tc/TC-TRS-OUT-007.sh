tc_TRS_OUT_007() {
    local F="$1"
    local M="$F/TC-TRS-OUT-007/export"

    # Scenario 1: valid JSON with schemaVersion + elements array.
    SCENARIO_NAME="export emits versioned JSON document"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out; out=$("$SYSCRIBE" -m "$M" export 2>/dev/null)
    local sv; sv=$(printf '%s' "$out" | jq -r '.schemaVersion')
    [ "$sv" = "1.0" ] && pass "schemaVersion is 1.0" || fail "schemaVersion='$sv' (expected 1.0)"
    local n; n=$(printf '%s' "$out" | jq -r '.elements | length')
    [ "$n" -ge 2 ] && pass "elements array has $n entries" || fail "elements length=$n (expected >=2)"

    # Scenario 2: requirement exposes computed.verifiedBy.
    SCENARIO_NAME="requirement exposes resolved verifiedBy"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local vb; vb=$(printf '%s' "$out" | jq -r '.elements[] | select(.id=="REQ-EXP-001") | .computed.verifiedBy[0]')
    [ "$vb" = "TC-EXP-001" ] && pass "REQ-EXP-001 verifiedBy TC-EXP-001" || fail "verifiedBy='$vb' (expected TC-EXP-001)"

    # Scenario 3: TestCase round-trips its verifies list.
    SCENARIO_NAME="testcase round-trips verifies"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local vf; vf=$(printf '%s' "$out" | jq -r '.elements[] | select(.id=="TC-EXP-001") | .frontmatter.verifies[0]')
    [ "$vf" = "REQ-EXP-001" ] && pass "TC-EXP-001 verifies REQ-EXP-001" || fail "verifies='$vf' (expected REQ-EXP-001)"

    # Scenario 4: NDJSON header line. Capture fully first, then take line 1 via
    # bash expansion (avoids a SIGPIPE that `| head -1` would raise under pipefail).
    SCENARIO_NAME="ndjson emits header then elements"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local nd; nd=$("$SYSCRIBE" -m "$M" export --ndjson 2>/dev/null)
    local first="${nd%%$'\n'*}"
    local hdr; hdr=$(printf '%s' "$first" | jq -r '.kind')
    [ "$hdr" = "header" ] && pass "first NDJSON line is header" || fail "first line kind='$hdr' (expected header)"
}
