tc_TRS_MCP_049() {
    local F="$1"; local B="$F/TC-TRS-MCP-049"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local W res

    _scn "an external edit is visible without calling reload"
    W=$(mktemp -d); cp -r "$B/model" "$W/model"
    res=$(python3 "$B/mcp_watch.py" "$SYSCRIBE" "$W/model" "$W/model/Requirements/REQ-WATCH-001.md" \
        "Original name" "Edited on disk" 2>&1) || true
    jq -e '.before == "Original name"' <<<"$res" >/dev/null 2>&1 && pass "initial name served" \
        || fail "initial read failed: $res"
    jq -e '.after == "Edited on disk"' <<<"$res" >/dev/null 2>&1 && pass "the edit is picked up automatically" \
        || fail "edit not picked up: $res"
    jq -e '.watchReloads >= 1' <<<"$res" >/dev/null 2>&1 && pass "a watch reload logging message is sent" \
        || fail "no watch reload logged: $res"
    jq -e '.listChanged' <<<"$res" >/dev/null 2>&1 && pass "resources/list_changed is sent" \
        || fail "no list_changed: $res"
    jq -e '.exited' <<<"$res" >/dev/null 2>&1 && pass "the server exits when stdin closes" \
        || fail "server did not exit: $res"
    rm -rf "$W"

    _scn "--no-watch keeps the loaded model"
    W=$(mktemp -d); cp -r "$B/model" "$W/model"
    res=$(python3 "$B/mcp_watch.py" "$SYSCRIBE" "$W/model" "$W/model/Requirements/REQ-WATCH-001.md" \
        "Original name" "Edited on disk" --no-watch 2>&1) || true
    jq -e '.after == "Original name" and .watchReloads == 0' <<<"$res" >/dev/null 2>&1 \
        && pass "no automatic reload under --no-watch" || fail "reloaded under --no-watch: $res"
    jq -e '.exited' <<<"$res" >/dev/null 2>&1 && pass "the server exits when stdin closes" \
        || fail "server did not exit: $res"
    rm -rf "$W"
}
