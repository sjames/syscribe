tc_TRS_SET_002() {
    local F="$1"; local FX="$F/TC-TRS-SET-002"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    local tmp M err rc
    tmp=$(mktemp -d)
    # fresh <name> — a scratch copy of the model (set writes files in place)
    fresh() { M="$tmp/$1"; cp -r "$FX/model" "$M"; }
    local PI_REL="Planning/PI-SET-002.md" ORIG="$FX/model/Planning/PI-SET-002.md"
    # setpi <args...> — run `set PI-SET-002 ...` from / (so relative paths can
    # only resolve against the model root); stderr -> $err, exit -> $rc
    setpi() { err=$(cd / && "$SYSCRIBE" -m "$M" set PI-SET-002 "$@" 2>&1 >/dev/null) && rc=0 || rc=$?; }
    # fm_list <file> <key> — block-list items (either indentation) of a frontmatter key
    fm_list() { awk -v k="$2" '/^---/{n++; if(n==2)exit; next} n==1 && $0 ~ "^"k":"{f=1;next} f && /^ *- /{sub(/^ *- /,"");print;next} f && /^[^ -]/{f=0}' "$1"; }

    # 1. achieves.add refuses a dangling target
    _scn "achieves.add refuses a dangling target"
    fresh s1; setpi achieves.add REQ-NONEXISTENT
    [ "$rc" -ne 0 ] && pass "exit code $rc (non-zero)" || fail "exit code 0 (expected non-zero)"
    printf '%s' "$err" | grep -qF "achieves 'REQ-NONEXISTENT' on Planning::PI-SET-002 — it does not resolve to any model element" \
        && pass "refusal reports the dangling target" || fail "unexpected refusal text: $err"
    cmp -s "$ORIG" "$M/$PI_REL" && pass "file unchanged" || fail "file modified by a refused achieves.add"

    # 2. achieves.add refuses a non-Requirement target
    _scn "achieves.add refuses a non-Requirement target"
    fresh s2; setpi achieves.add Arch::PartX
    [ "$rc" -ne 0 ] && pass "exit code $rc (non-zero)" || fail "exit code 0 (expected non-zero)"
    printf '%s' "$err" | grep -qF "achieves 'Arch::PartX' on Planning::PI-SET-002 — it does not resolve to a native Requirement" \
        && pass "refusal names that the target must be a native Requirement" || fail "unexpected refusal text: $err"
    cmp -s "$ORIG" "$M/$PI_REL" && pass "file unchanged" || fail "file modified by a refused achieves.add"

    # 3. achieves.add appends after existing entries
    _scn "achieves.add appends after existing entries"
    fresh s3; setpi achieves.add REQ-SET-011
    [ "$rc" -eq 0 ] && pass "exit code 0" || fail "exit code $rc (expected 0): $err"
    [ "$(fm_list "$M/$PI_REL" achieves | paste -sd,)" = "REQ-SET-010,REQ-SET-011" ] \
        && pass "achieves: is exactly [REQ-SET-010, REQ-SET-011], in that order" \
        || fail "achieves: order wrong: $(fm_list "$M/$PI_REL" achieves | paste -sd,)"

    # 4. evidence.add ref must resolve
    _scn "evidence.add ref must resolve"
    fresh s4
    grep -q '^evidence:' "$ORIG" && fail "precondition: fixture PI already has evidence" || pass "precondition: PI has no evidence"
    setpi evidence.add ref=NONEXISTENT
    [ "$rc" -ne 0 ] && pass "exit code $rc (non-zero)" || fail "exit code 0 (expected non-zero)"
    printf '%s' "$err" | grep -qF "evidence ref='NONEXISTENT' on Planning::PI-SET-002 — it does not resolve to any model element" \
        && pass "refusal reports the dangling ref" || fail "unexpected refusal text: $err"
    cmp -s "$ORIG" "$M/$PI_REL" && pass "file unchanged" || fail "file modified by a refused evidence.add"

    # 5. evidence.add path must exist on disk or be a remote URI
    _scn "evidence.add path must exist on disk or be a remote URI"
    fresh s5n; setpi evidence.add path=src/nope.rs
    [ "$rc" -ne 0 ] && pass "nonexistent local path: exit code $rc (non-zero)" || fail "nonexistent local path accepted (exit 0)"
    printf '%s' "$err" | grep -qF "evidence path='src/nope.rs' on Planning::PI-SET-002 — it does not exist on disk" \
        && pass "refusal reports the missing path" || fail "unexpected refusal text: $err"
    cmp -s "$ORIG" "$M/$PI_REL" && pass "file unchanged for the nonexistent path" || fail "file modified by a refused evidence.add path"
    fresh s5e; setpi evidence.add path=src/real.rs
    [ "$rc" -eq 0 ] && pass "existing local path (model-root relative): exit code 0" || fail "existing local path refused (exit $rc): $err"
    fm_list "$M/$PI_REL" evidence | grep -qxF 'path: src/real.rs' \
        && pass "evidence entry 'path: src/real.rs' appended" || fail "evidence entry for src/real.rs not written"
    fresh s5u; setpi evidence.add path=https://example.com/report.html
    [ "$rc" -eq 0 ] && pass "https:// URI: exit code 0" || fail "https:// URI refused (exit $rc): $err"
    fm_list "$M/$PI_REL" evidence | grep -qxF 'path: https://example.com/report.html' \
        && pass "evidence entry for the https:// URI appended" || fail "evidence entry for the URI not written"

    # setpi_out <args...> — like setpi, but stdout -> $out (no-op report and diffs go to stdout)
    local out
    setpi_out() { out=$(cd / && "$SYSCRIBE" -m "$M" set PI-SET-002 "$@" 2>/dev/null) && rc=0 || rc=$?; }
    # ev_block <file> — the raw evidence: block of the frontmatter (entries + continuation lines)
    ev_block() { awk '/^---/{n++; if(n==2)exit; next} n==1 && /^evidence:/{f=1;next} f && /^[^ -]/{f=0} f' "$1"; }

    # 6. achieves.add of an already-present id is a no-op
    _scn "achieves.add of an already-present id is a no-op"
    fresh s6; setpi_out achieves.add REQ-SET-010
    [ "$rc" -eq 0 ] && pass "exit code 0" || fail "exit code $rc (expected 0)"
    printf '%s' "$out" | grep -qF "already achieves 'REQ-SET-010' — nothing to do" \
        && pass "reports the id is already present" || fail "no already-present report: $out"
    [ "$(fm_list "$M/$PI_REL" achieves | grep -cxF 'REQ-SET-010' || true)" -eq 1 ] \
        && pass "REQ-SET-010 appears exactly once in achieves:" || fail "REQ-SET-010 duplicated or lost: $(fm_list "$M/$PI_REL" achieves | paste -sd,)"
    cmp -s "$ORIG" "$M/$PI_REL" && pass "file byte-identical" || fail "file modified by a no-op achieves.add"

    # 7. evidence.add refuses both ref and path, or neither
    _scn "evidence.add refuses both ref and path, or neither"
    fresh s7b; setpi evidence.add ref=REQ-SET-011 path=src/real.rs
    [ "$rc" -ne 0 ] && pass "both ref= and path=: exit code $rc (non-zero)" || fail "both ref= and path= accepted (exit 0)"
    printf '%s' "$err" | grep -qF 'pass ref=<id> or path=<path>, not both' \
        && pass "refusal says not both" || fail "unexpected refusal text: $err"
    cmp -s "$ORIG" "$M/$PI_REL" && pass "file unchanged (both)" || fail "file modified when both ref= and path= were given"
    fresh s7n; setpi evidence.add rationale=orphan
    [ "$rc" -ne 0 ] && pass "neither ref= nor path=: exit code $rc (non-zero)" || fail "neither ref= nor path= accepted (exit 0)"
    printf '%s' "$err" | grep -qF 'evidence.add requires ref=<id> or path=<path>' \
        && pass "refusal says ref= or path= is required" || fail "unexpected refusal text: $err"
    cmp -s "$ORIG" "$M/$PI_REL" && pass "file unchanged (neither)" || fail "file modified when neither ref= nor path= was given"

    # 8. evidence.add carries an optional rationale onto the new entry
    _scn "evidence.add carries an optional rationale onto the new entry"
    local ev
    fresh s8r; setpi evidence.add ref=REQ-SET-011 "rationale=Reviewed offline by QA: see minutes"
    [ "$rc" -eq 0 ] && pass "exit code 0" || fail "exit code $rc (expected 0): $err"
    ev=$(ev_block "$M/$PI_REL")
    [ "$(printf '%s\n' "$ev" | grep -cE '^ *- ' || true)" -eq 1 ] \
        && pass "exactly one evidence entry" || fail "expected exactly one evidence entry: $ev"
    printf '%s\n' "$ev" | grep -qE '^ *(- )? *ref: REQ-SET-011$' \
        && pass "entry carries ref: REQ-SET-011" || fail "entry lacks the ref: $ev"
    printf '%s\n' "$ev" | grep -qE "^ *(- )? *rationale: (['\"]?)Reviewed offline by QA: see minutes\\2\$" \
        && pass "entry carries the full rationale text (spaces and colon intact)" || fail "rationale missing or altered: $ev"
    fresh s8n; setpi evidence.add path=src/real.rs
    ev=$(ev_block "$M/$PI_REL")
    [ "$rc" -eq 0 ] && printf '%s\n' "$ev" | grep -qE '^ *(- )? *path: src/real.rs$' && ! printf '%s\n' "$ev" | grep -q 'rationale:' \
        && pass "an entry added without rationale= carries no rationale" || fail "unexpected rationale (or entry missing, exit $rc): $ev"

    # 9. --dry-run previews achieves.add and evidence.add without writing
    _scn "--dry-run previews achieves.add and evidence.add without writing"
    fresh s9a; setpi_out achieves.add REQ-SET-011 --dry-run
    [ "$rc" -eq 0 ] && pass "achieves.add --dry-run: exit code 0" || fail "achieves.add --dry-run: exit code $rc"
    printf '%s\n' "$out" | grep -qxF -- '--- a/Planning/PI-SET-002.md' && printf '%s\n' "$out" | grep -qxF -- '+++ b/Planning/PI-SET-002.md' \
        && printf '%s\n' "$out" | grep -qE '^@@ -[0-9]+(,[0-9]+)? \+[0-9]+(,[0-9]+)? @@' \
        && pass "achieves.add: unified diff headers and hunk printed" || fail "achieves.add: no unified diff: $out"
    printf '%s\n' "$out" | grep -qE '^\+ *- REQ-SET-011$' \
        && pass "achieves.add: diff adds REQ-SET-011" || fail "achieves.add: diff lacks the added requirement: $out"
    cmp -s "$ORIG" "$M/$PI_REL" && pass "achieves.add --dry-run: file unchanged" || fail "achieves.add --dry-run wrote the file"
    fresh s9e; setpi_out evidence.add path=src/real.rs --dry-run
    [ "$rc" -eq 0 ] && pass "evidence.add --dry-run: exit code 0" || fail "evidence.add --dry-run: exit code $rc"
    printf '%s\n' "$out" | grep -qxF -- '--- a/Planning/PI-SET-002.md' && printf '%s\n' "$out" | grep -qxF -- '+++ b/Planning/PI-SET-002.md' \
        && printf '%s\n' "$out" | grep -qE '^@@ -[0-9]+(,[0-9]+)? \+[0-9]+(,[0-9]+)? @@' \
        && pass "evidence.add: unified diff headers and hunk printed" || fail "evidence.add: no unified diff: $out"
    printf '%s\n' "$out" | grep -qxF '+evidence:' && printf '%s\n' "$out" | grep -qE '^\+ *- path: src/real.rs$' \
        && pass "evidence.add: diff adds the evidence entry" || fail "evidence.add: diff lacks the evidence entry: $out"
    cmp -s "$ORIG" "$M/$PI_REL" && pass "evidence.add --dry-run: file unchanged" || fail "evidence.add --dry-run wrote the file"

    rm -rf "$tmp"
}
