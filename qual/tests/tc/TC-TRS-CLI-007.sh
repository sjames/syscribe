tc_TRS_CLI_007() {
    local out rc
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # Expected version = the syscribe crate package version.
    local ver
    ver=$(awk '/^\[package\]/{p=1} p&&/^version *=/{gsub(/[" ]/,""); split($0,a,"="); print a[2]; exit}' \
        "$REPO_ROOT/crates/syscribe/Cargo.toml")
    [ -n "$ver" ] || ver=$(grep -m1 '^version' "$REPO_ROOT/crates/syscribe/Cargo.toml" | sed 's/[^0-9.]//g')

    _scn "--version prints 'syscribe <semver>' and exits 0"
    out=$("$SYSCRIBE" --version 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qx "syscribe $ver"; } \
        && pass "--version -> 'syscribe $ver', exit 0" || fail "--version wrong (rc=$rc, out='$out', want 'syscribe $ver')"

    _scn "-V prints 'syscribe <semver>' and exits 0"
    out=$("$SYSCRIBE" -V 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qx "syscribe $ver"; } \
        && pass "-V -> 'syscribe $ver', exit 0" || fail "-V wrong (rc=$rc, out='$out')"

    _scn "version subcommand prints 'syscribe <semver>' and exits 0"
    out=$("$SYSCRIBE" version 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qx "syscribe $ver"; } \
        && pass "version -> 'syscribe $ver', exit 0" || fail "version subcommand wrong (rc=$rc, out='$out')"

    _scn "version after a model flag behaves like 'syscribe version' (issue #133)"
    local MD="$REPO_ROOT/qual/fixtures/TC-TRS-CFLD-002/model" spelling
    for spelling in "-m $MD version" "--model $MD version" "--model=$MD version"; do
        # shellcheck disable=SC2086
        out=$("$SYSCRIBE" $spelling 2>&1) && rc=0 || rc=$?
        { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qx "syscribe $ver"; } \
            && pass "${spelling%% *} <root> version -> 'syscribe $ver', exit 0" \
            || fail "${spelling%% *} <root> version wrong (rc=$rc, out='$(head -c 160 <<<"$out")')"
    done

    _scn "works from a directory with no model or .syscribe.toml"
    local W; W=$(mktemp -d)
    out=$(cd "$W" && "$SYSCRIBE" --version 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 0 ] && printf '%s' "$out" | grep -qx "syscribe $ver"; } \
        && pass "version prints without a model directory" || fail "failed with no model dir (rc=$rc, out='$out')"
    rm -rf "$W"
}
