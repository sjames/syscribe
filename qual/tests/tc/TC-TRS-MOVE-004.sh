tc_TRS_MOVE_004() {
    local F="$1"; local BASE="$F/move/base"
    local W

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    # Move the native Requirement to a new qualified name.
    "$SYSCRIBE" -m "$W" move Reqs::REQ Moved::REQ >/dev/null 2>&1 || true

    _scn "stable id is unchanged after move"
    { [ -f "$W/Moved/REQ.md" ] && grep -qF "id: REQ-MV-001" "$W/Moved/REQ.md"; } \
        && pass "REQ-MV-001 id preserved at new path" || fail "id changed or file missing"

    _scn "id-based reference is not rewritten and still resolves"
    grep -qF "REQ-MV-001" "$W/Reqs/TC.md" && pass "verifies still reads REQ-MV-001" || fail "verifies entry was altered"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$W" validate 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_no_code "E102"
    rm -rf "$W"
}
