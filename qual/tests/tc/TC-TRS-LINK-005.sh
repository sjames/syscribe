tc_TRS_LINK_005() {
    # $1 = fixtures dir (unused: the behaviour lives in the live web server and
    # is exercised by an in-process Axum integration test in the syscribe-server
    # crate). The repo root is two levels up from the fixtures dir.
    local REPO_ROOT; REPO_ROOT="$(cd "$1/../.." && pwd)"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "live detail panel renders/omits the source-link icon per [links]"
    if ( cd "$REPO_ROOT" && cargo test -p syscribe-server --test source_link link_005 >/dev/null 2>&1 ); then
        pass "detail-panel source-link integration test passes (presence + absence)"
    else
        fail "detail-panel source-link integration test failed"
    fi
}
