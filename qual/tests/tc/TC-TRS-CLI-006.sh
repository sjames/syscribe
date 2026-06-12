tc_TRS_CLI_006() {
    # The flag needs no model directory; run from a tmp cwd to prove it.
    local out rc
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. magicgrid topic prints the MagicGrid prompt
    _scn "magicgrid topic prints the MagicGrid modeling prompt"
    out=$("$SYSCRIBE" --agent-instructions magicgrid 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] \
        && printf '%s' "$out" | grep -q 'Modeling with MagicGrid' \
        && printf '%s' "$out" | grep -q 'mg_cell' \
        && printf '%s' "$out" | grep -q 'magicgrid --audit' \
        && printf '%s' "$out" | grep -q 'trade-study'; } \
        && pass "MagicGrid prompt printed, exit 0" || fail "magicgrid prompt missing/incorrect (rc=$rc)"

    # 2. no topic prints the general prompt (not the MagicGrid one)
    _scn "no topic prints the general modeling prompt"
    out=$("$SYSCRIBE" --agent-instructions 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && ! printf '%s' "$out" | grep -q 'Modeling with MagicGrid'; } \
        && pass "general prompt printed, no MagicGrid heading" || fail "no-topic output wrong (rc=$rc)"

    # 3. an unknown topic exits non-zero naming the topics
    _scn "an unknown topic exits non-zero naming the topics"
    out=$("$SYSCRIBE" --agent-instructions wibble 2>&1) && rc=0 || rc=$?
    { [ "$rc" -ne 0 ] && printf '%s' "$out" | grep -qi 'magicgrid'; } \
        && pass "unknown topic exits non-zero, names magicgrid" || fail "unknown topic not rejected (rc=$rc)"

    # 4. works without a model directory (run from an empty tmp dir)
    _scn "works without a model directory"
    local W; W=$(mktemp -d)
    out=$(cd "$W" && "$SYSCRIBE" --agent-instructions magicgrid 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -q 'Modeling with MagicGrid'; } \
        && pass "prints without a model directory" || fail "failed with no model dir (rc=$rc)"
    rm -rf "$W"
}
