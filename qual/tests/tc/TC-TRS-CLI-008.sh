tc_TRS_CLI_008() {
    local F="$1"; local out rc
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    local MODEL="$F/TC-TRS-CLI-004/root"   # reuse: a model containing PartDef Engine

    _scn "unknown command exits non-zero from a dir with no model (not a model-path error)"
    local W; W=$(mktemp -d)
    out=$(cd "$W" && unset SYSCRIBE_MODEL; "$SYSCRIBE" bogus-command 2>&1) && rc=0 || rc=$?
    rm -rf "$W"
    { [ "$rc" -ne 0 ] \
        && ! printf '%s' "$out" | grep -qi 'model path' \
        && printf '%s' "$out" | grep -qiE 'bogus-command|unrecognized|unexpected|usage|error'; } \
        && pass "unknown command rejected (rc=$rc), no model-path error" \
        || fail "unknown command not cleanly rejected (rc=$rc, out='$out')"

    _scn "a registered command still runs and exits 0"
    out=$("$SYSCRIBE" -m "$MODEL" list PartDef 2>/dev/null) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qF "Engine"; } \
        && pass "list PartDef lists Engine, exit 0" || fail "registered command broke (rc=$rc)"

    _scn "man-page help is preserved: validate --help prints SYNOPSIS, exit 0"
    out=$("$SYSCRIBE" validate --help 2>/dev/null) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qF "SYNOPSIS"; } \
        && pass "validate --help -> SYNOPSIS man-page, exit 0" || fail "man-page help regressed (rc=$rc)"

    _scn "version is preserved: --version prints 'syscribe <semver>', exit 0"
    local ver; ver=$(grep -m1 '^version' "$REPO_ROOT/crates/syscribe/Cargo.toml" | sed 's/[^0-9.]//g')
    out=$("$SYSCRIBE" --version 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qx "syscribe $ver"; } \
        && pass "--version -> 'syscribe $ver'" || fail "version regressed (rc=$rc, out='$out')"

    _scn "explicit 'report' runs the default validation report"
    out=$("$SYSCRIBE" -m "$MODEL" report 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && ! printf '%s' "$out" | grep -qi 'unknown command'; } \
        && pass "report runs the default report (exit 0)" || fail "report not wired (rc=$rc, out='$(printf '%s' "$out" | head -1)')"
}
