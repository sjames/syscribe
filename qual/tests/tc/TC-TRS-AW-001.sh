tc_TRS_AW_001() {
    local F="$1"; local BASE="$F/TC-TRS-AW-001/base"; local BAD="$F/TC-TRS-AW-001/badmodel"
    local W rc out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. set by FEATURE ID
    _scn "set appliesWhen by feature id writes the field"
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    out=$("$SYSCRIBE" -m "$W" applies-when REQ-AW-BRAKE-001 --set "FEAT-ABS" 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && grep -qx 'appliesWhen: FEAT-ABS' "$W/Reqs/Brake.md"; } \
        && pass "appliesWhen: FEAT-ABS written by id" || fail "set by feature id failed (rc=$rc)"
    rm -rf "$W"

    # 2. set by FEATURE PATH (qualified name)
    _scn "set appliesWhen by feature path (qualified name) writes the field"
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    out=$("$SYSCRIBE" -m "$W" applies-when REQ-AW-BRAKE-001 --set "Features::Abs" 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && grep -qx 'appliesWhen: Features::Abs' "$W/Reqs/Brake.md"; } \
        && pass "appliesWhen: Features::Abs written by path" || fail "set by feature path failed (rc=$rc)"
    rm -rf "$W"

    # 3. an unresolved feature is refused (E209) and nothing is written
    _scn "an unknown feature operand is refused with E209 and no write"
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    out=$("$SYSCRIBE" -m "$W" applies-when REQ-AW-BRAKE-001 --set "FEAT-NOPE" 2>&1) && rc=0 || rc=$?
    { [ "$rc" -ne 0 ] && printf '%s' "$out" | grep -q 'E209' && ! grep -q '^appliesWhen:' "$W/Reqs/Brake.md"; } \
        && pass "E209 refusal, file unchanged" || fail "unknown feature was not refused cleanly (rc=$rc)"
    rm -rf "$W"

    # 4. a forbidden target (a FeatureDef) is refused (E228) and nothing is written
    _scn "setting appliesWhen on a FeatureDef is refused with E228"
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    out=$("$SYSCRIBE" -m "$W" applies-when Features::Abs --set "FEAT-ESC" 2>&1) && rc=0 || rc=$?
    { [ "$rc" -ne 0 ] && printf '%s' "$out" | grep -q 'E228' && ! grep -q '^appliesWhen:' "$W/Features/Abs.md"; } \
        && pass "E228 refusal on FeatureDef, file unchanged" || fail "FeatureDef target not refused (rc=$rc)"
    rm -rf "$W"

    # 5. on a void feature model the gate is written but the bad-config check exits non-zero
    _scn "a void feature model is reported (E223) with a non-zero exit after set"
    W=$(mktemp -d); cp -r "$BAD"/. "$W"/
    out=$("$SYSCRIBE" -m "$W" applies-when REQ-AW-VOID-001 --set "FEAT-AA" 2>&1) && rc=0 || rc=$?
    { [ "$rc" -ne 0 ] && printf '%s' "$out" | grep -q 'E223'; } \
        && pass "void model reported (E223), non-zero exit" || fail "void model not reported on set (rc=$rc)"
    rm -rf "$W"

    # 6. --clear removes the field
    _scn "clear removes an existing appliesWhen"
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    "$SYSCRIBE" -m "$W" applies-when REQ-AW-BRAKE-001 --set "FEAT-ABS" >/dev/null 2>&1 || true
    "$SYSCRIBE" -m "$W" applies-when REQ-AW-BRAKE-001 --clear >/dev/null 2>&1 && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && ! grep -q '^appliesWhen:' "$W/Reqs/Brake.md"; } \
        && pass "clear removed the field" || fail "clear did not remove the field (rc=$rc)"
    rm -rf "$W"

    # 7. --dry-run writes nothing
    _scn "dry-run validates but writes nothing"
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    "$SYSCRIBE" -m "$W" applies-when REQ-AW-BRAKE-001 --set "FEAT-ABS" --dry-run >/dev/null 2>&1 || true
    grep -q '^appliesWhen:' "$W/Reqs/Brake.md" \
        && fail "dry-run wrote the field" || pass "dry-run left the file unchanged"
    rm -rf "$W"
}
