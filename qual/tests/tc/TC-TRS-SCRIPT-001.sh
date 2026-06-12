tc_TRS_SCRIPT_001() {
    local F="$1"
    local M="$F/TC-TRS-SCRIPT-001"
    # A model with no scripts directory.
    local NONE; NONE=$(mktemp -d)
    printf '[scripts]\npath = ".syscribe/scripts"\n' > "$NONE/.syscribe.toml"
    printf -- '---\ntype: Package\nname: Empty\n---\nEmpty.\n' > "$NONE/_index.md"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "discover scripts and reuse a library module"
    local out rc=0; out=$("$SYSCRIBE" -m "$M" scripts run summary 2>/dev/null) || rc=$?
    [ "$rc" -eq 0 ] && pass "scripts run summary exit 0" || fail "scripts run summary exit $rc"
    printf '%s' "$out" | grep -qF "== Requirement summary ==" \
        && pass "library banner() output present (import reuse)" \
        || fail "library banner() output missing"
    printf '%s' "$out" | grep -qF "requirements: 2" \
        && pass "library count_reqs() reused over the model" \
        || fail "library count_reqs() output missing"

    _scn "scripts list enumerates the registered command"
    local lst; lst=$("$SYSCRIBE" -m "$M" scripts list 2>/dev/null) || true
    printf '%s' "$lst" | grep -qF "summary" \
        && pass "summary command listed" || fail "summary command not listed"

    _scn "scripts are not surfaced as model elements"
    local le; le=$("$SYSCRIBE" -m "$M" list 2>/dev/null) || true
    printf '%s' "$le" | grep -qiF ".rhai" \
        && fail "a .rhai script leaked into the element list" \
        || pass "no .rhai script in the element list"
    printf '%s' "$le" | grep -qiF "summary" \
        && fail "summary script leaked into the element list" \
        || pass "summary script not a model element"

    _scn "a model with no scripts directory runs normally"
    local n nrc=0; n=$("$SYSCRIBE" -m "$NONE" scripts list 2>/dev/null) || nrc=$?
    [ "$nrc" -eq 0 ] && pass "scripts list exit 0 with no scripts dir" || fail "exit $nrc"
    printf '%s' "$n" | grep -qiF "no" \
        && pass "reports that none are defined" || fail "did not report none defined"

    rm -rf "$NONE"
}
