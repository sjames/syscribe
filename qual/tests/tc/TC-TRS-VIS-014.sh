tc_TRS_VIS_014() {
    local R="$REPO_ROOT"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "every visualisation layer has a non-empty test file"
    for f in \
        crates/syscribe-model/src/vis/manifest.rs \
        crates/syscribe-model/tests/vis_manifest_validation.rs \
        crates/syscribe-model/tests/vis_derive.rs \
        crates/syscribe-model/tests/vis_derive_behaviour.rs \
        crates/syscribe-model/tests/vis_derive_trace.rs \
        crates/syscribe-model/tests/vis_derive_sequence.rs \
        crates/syscribe-model/tests/vis_writers.rs \
        crates/syscribe-model/tests/vis_plantuml_snapshot.rs \
        crates/syscribe-model/tests/vis_layout.rs \
        crates/syscribe-model/tests/vis_elk_vendor.rs \
        crates/syscribe-server/tests/diagram_model.rs \
        crates/syscribe-server/tests/layout_routes.rs \
        crates/syscribe-server/frontend/test/elk-layout.test.mjs \
        crates/syscribe-server/frontend/test/connect-rules.test.mjs \
        crates/syscribe-server/frontend/test/server-sizes.test.mjs; do
        [ -s "$R/$f" ] && grep -qE "#\[test\]|#\[tokio::test\]|assert|scenario\(" "$R/$f" \
            && pass "$f has tests" || fail "$f missing or has no tests"
    done

    _scn "golden snapshots exist for the IR, the writers and the ELK determinism check"
    for d in derived writers elk; do
        n=$(ls "$R/crates/syscribe-model/tests/vis_snapshots/$d" 2>/dev/null | wc -l)
        [ "$n" -gt 0 ] && pass "vis_snapshots/$d holds $n file(s)" || fail "vis_snapshots/$d is empty"
    done
    [ -s "$R/crates/syscribe-model/tests/vis_snapshots/elk/power_ibd.output.json" ] \
        && pass "Node-produced ELK output committed for the determinism test" \
        || fail "elk/power_ibd.output.json missing"

    _scn "the frontend test script runs every Node test"
    local pkg="$R/crates/syscribe-server/frontend/package.json"
    for t in elk-layout.test.mjs connect-rules.test.mjs server-sizes.test.mjs; do
        grep -q "test/$t" "$pkg" && pass "npm test runs $t" || fail "npm test does not run $t"
    done
}
