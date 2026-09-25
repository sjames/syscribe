tc_TRS_VAR_008() {
    local F="$1"; local B="$F/TC-TRS-VAR-008"; local M="$B/model"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out res

    _scn "the fixture validates cleanly"
    out=$("$SYSCRIBE" -m "$M" validate 2>/dev/null) || true
    grep -qF "0 errors, 0 warnings" <<<"$out" && pass "no findings" || fail "fixture is not clean: $out"

    _scn "renaming the base id is accepted and edits base and child"
    # Line 2 (0-based) of the base file is `id: CONF-VR-BASE-001`.
    res=$(python3 "$B/lsp_rename.py" "$SYSCRIBE" "$M" "$M/Configurations/CONF-VR-BASE-001.md" 2 8 CONF-VR-BASE-002 2>&1) || true
    jq -e '.result.changes' <<<"$res" >/dev/null 2>&1 && pass "rename returns a WorkspaceEdit" \
        || fail "rename refused or failed: $res"
    jq -e '.result.changes | to_entries | any(.key | endswith("CONF-VR-BASE-001.md"))' <<<"$res" >/dev/null 2>&1 \
        && pass "edits the base's file" || fail "base file not edited: $res"
    jq -e '.result.changes | to_entries | any(.[]; (.key | endswith("CONF-VR-CHILD-001.md"))
            and any(.value[]; .newText == "derivedFrom: CONF-VR-BASE-002"))' <<<"$res" >/dev/null 2>&1 \
        && pass "edits the child's derivedFrom to the new id" || fail "child derivedFrom not edited: $res"
}
