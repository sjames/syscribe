tc_TRS_SET_003() {
    local F="$1"; local FX="$F/TC-TRS-SET-003"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }

    local tmp M out rc
    tmp=$(mktemp -d)
    local PI_REL="Planning/PI-SET-003.md" ORIG="$FX/model/Planning/PI-SET-003.md"
    fresh() { M="$tmp/$1"; cp -r "$FX/model" "$M"; }
    # setpi <args...> — run `set PI-SET-003 ...`; stdout -> $out, exit -> $rc
    setpi() { out=$(cd / && "$SYSCRIBE" -m "$M" set PI-SET-003 "$@" 2>&1) && rc=0 || rc=$?; }
    # removed_lines <file> — lines of the original missing from <file> (as a multiset)
    removed_lines() { diff -- "$ORIG" "$1" | grep -E '^< ' || true; }
    local c1="# Tracked in the Q3 plan — keep this comment." c2="  # Proof collected so far." c3="# End of planning fields."

    _scn "evidence.add of an existing ref is a reported no-op"
    fresh s1; setpi evidence.add ref=REQ-SET-011
    [ "$rc" -eq 0 ] && pass "exit code 0" || fail "exit code $rc (expected 0): $out"
    grep -qF "already has evidence ref='REQ-SET-011' — nothing to do" <<<"$out" \
        && pass "reports the entry is already present" || fail "no already-present report: $out"
    cmp -s "$ORIG" "$M/$PI_REL" && pass "file byte-identical" || fail "file modified by a no-op evidence.add: $(diff -- "$ORIG" "$M/$PI_REL" || true)"

    _scn "evidence.add of an existing path is a reported no-op"
    fresh s2; setpi evidence.add path=src/real.rs
    [ "$rc" -eq 0 ] && pass "exit code 0" || fail "exit code $rc (expected 0): $out"
    grep -qF "already has evidence path='src/real.rs' — nothing to do" <<<"$out" \
        && pass "reports the entry is already present" || fail "no already-present report: $out"
    cmp -s "$ORIG" "$M/$PI_REL" && pass "file byte-identical" || fail "file modified by a no-op evidence.add"

    _scn "evidence.add of a new entry preserves YAML comments and every other line"
    fresh s3; setpi evidence.add ref=REQ-SET-010 "rationale=Covered by review"
    [ "$rc" -eq 0 ] && pass "exit code 0" || fail "exit code $rc (expected 0): $out"
    local after; after=$(cat -- "$M/$PI_REL")
    for c in "$c1" "$c2" "$c3" "  - REQ-SET-010   # the parent requirement" "  - ref: REQ-SET-011  # reviewed" 'name: "An item with commented frontmatter"'; do
        grep -qxF -- "$c" <<<"$after" && pass "kept: $c" || fail "lost: $c"
    done
    local rm_lines; rm_lines=$(removed_lines "$M/$PI_REL")
    [ -z "$rm_lines" ] && pass "no original line removed or altered" || fail "original lines changed: $rm_lines"
    grep -qxF "  - ref: REQ-SET-010" <<<"$after" && grep -qE "^    rationale: (['\"]?)Covered by review\1$" <<<"$after" \
        && pass "new entry appended at the list's indentation" || fail "new entry missing or mis-indented: $after"
    local n; n=$(diff -- "$ORIG" "$M/$PI_REL" | grep -cE '^> ' || true)
    [ "${n:-0}" -eq 2 ] && pass "exactly the two new entry lines added" || fail "added $n lines (expected 2)"
    "$SYSCRIBE" -m "$M" validate >/dev/null 2>&1 && pass "model still validates" || fail "model no longer validates"

    _scn "achieves.add of a new requirement preserves YAML comments"
    fresh s4; setpi achieves.add REQ-SET-011
    [ "$rc" -eq 0 ] && pass "exit code 0" || fail "exit code $rc (expected 0): $out"
    after=$(cat -- "$M/$PI_REL")
    rm_lines=$(removed_lines "$M/$PI_REL")
    [ -z "$rm_lines" ] && pass "no original line removed or altered" || fail "original lines changed: $rm_lines"
    grep -qxF "  - REQ-SET-011" <<<"$after" && pass "REQ-SET-011 appended to achieves:" || fail "REQ-SET-011 not appended: $after"

    rm -rf "$tmp"
}
