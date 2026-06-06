tc_TRS_TAG_001() {
    local F="$1"; local M="$F/TC-TRS-TAG-001/model"

    # Scenario: list --tag selects only tagged elements.
    SCENARIO_NAME="list --tag selects only tagged elements"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local ls; ls=$("$SYSCRIBE" -m "$M" list Requirement --tag smoke 2>/dev/null || true)
    printf '%s' "$ls" | grep -qF "REQ-TAG-001" \
        && pass "smoke-tagged REQ-TAG-001 listed" || fail "REQ-TAG-001 not listed under --tag smoke"
    printf '%s' "$ls" | grep -qF "REQ-TAG-002" \
        && fail "untagged REQ-TAG-002 wrongly listed" || pass "REQ-TAG-002 excluded"

    # Scenario: an unknown tag is not an error.
    SCENARIO_NAME="unknown tag is not an error"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$M" list Requirement --tag nonexistent >/dev/null 2>&1 && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    [ "$SCENARIO_EXIT" -eq 0 ] && pass "unknown tag exits 0" || fail "exit $SCENARIO_EXIT (expected 0)"

    # Scenario: matrix --tag filters rows but keeps all columns.
    SCENARIO_NAME="matrix --tag filters rows but not columns"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local J; J=$("$SYSCRIBE" -m "$M" matrix --json --tag smoke 2>/dev/null || true)
    local rows; rows=$(printf '%s' "$J" | jq -r '[.rows[].id] | sort | join(",")' 2>/dev/null || true)
    [ "$rows" = "REQ-TAG-001" ] \
        && pass "only smoke-tagged requirement is a row" || fail "rows='$rows' (expected REQ-TAG-001)"
    local ncols; ncols=$(printf '%s' "$J" | jq -r '.columns | length' 2>/dev/null || true)
    [ "${ncols:-0}" -eq 2 ] && pass "both configurations remain as columns" || fail "columns=${ncols:-0} (expected 2)"
}
