tc_TRS_FMA_006() {
    local F="$1"
    local A="$F/TC-TRS-FMA-003/anomalies"

    # Generate a realistic feature model: a mandatory Root (core), an XOR group,
    # an OR group, two cross-tree constraints, and N plain optional features
    # (total = N + 7). Kept satisfiable. Echoes the temp dir.
    _gen_features() {
        local n="$1" dir; dir=$(mktemp -d)
        mkdir -p "$dir/Features/Alt" "$dir/Features/Or" "$dir/Configs"
        _f() { printf -- '---\ntype: FeatureDef\nname: %s\ngroupKind: %s\n%s---\n%s.\n' "$2" "$3" "$4" "$2" > "$dir/Features/$1.md"; }
        _f Root Root mandatory ""
        _f Alt/_index Alt alternative ""        # XOR group
        _f Alt/A1 A1 optional ""
        _f Alt/A2 A2 optional ""
        _f Or/_index Or or ""                    # OR group
        _f Or/O1 O1 optional ""
        _f Or/O2 O2 optional ""
        local i=1
        while [ "$i" -le "$n" ]; do
            local extra=""
            [ "$i" = "1" ] && extra=$'requires:\n  - Features::F2\n'    # cross-tree require
            [ "$i" = "3" ] && extra=$'excludes:\n  - Features::F4\n'    # cross-tree exclude
            printf -- '---\ntype: FeatureDef\nname: F%s\ngroupKind: optional\n%s---\nF%s.\n' "$i" "$extra" "$i" > "$dir/Features/F$i.md"
            i=$((i+1))
        done
        printf -- '---\ntype: Configuration\nid: CONF-GEN-001\ntitle: c\nstatus: approved\nfeatureModel: Features\nfeatures:\n  Features::Root: true\n  Features::Alt: true\n  Features::Alt::A1: true\n---\nc.\n' > "$dir/Configs/C1.md"
        printf '%s' "$dir"
    }

    # Scenario: determinism — two runs are byte-identical.
    SCENARIO_NAME="determinism: two runs are identical"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local a b
    a=$("$SYSCRIBE" -m "$A" feature-check --deep --json 2>/dev/null || true)
    b=$("$SYSCRIBE" -m "$A" feature-check --deep --json 2>/dev/null || true)
    [ "$a" = "$b" ] && pass "deep --json output identical across runs" || fail "non-deterministic output"

    # Scenario: ~500 features is analyzed (not skipped), correct, within interactive time.
    SCENARIO_NAME="~500-feature scale is analyzed within interactive time"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local scale; scale=$(_gen_features 493)   # 493 plain + 7 structured = 500 features
    local j; j=$(timeout 30 "$SYSCRIBE" -m "$scale" feature-check --deep --json 2>/dev/null) && ec=0 || ec=$?
    [ "${ec:-0}" -ne 124 ] && pass "500 features completed under the 30s timeout" || fail "500-feature analysis timed out"
    printf '%s' "$j" | jq -e '.deepSkipped == null' >/dev/null 2>&1 \
        && pass "500 features not skipped by the guard" || fail "500 features wrongly skipped"
    printf '%s' "$j" | jq -e '.void == false' >/dev/null 2>&1 \
        && pass "500-feature model is not void" || fail "500-feature void verdict wrong"
    printf '%s' "$j" | jq -e '.coreFeatures | index("Features::Root")' >/dev/null 2>&1 \
        && pass "mandatory Root detected core at scale" || fail "core detection wrong at scale"
    rm -rf "$scale"

    # Scenario: above the limit (1000), the guard skips gracefully.
    SCENARIO_NAME="size guard skips above the documented limit"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local big; big=$(_gen_features 1001)   # 1001 plain + 7 structured = 1008 > 1000
    local out; out=$("$SYSCRIBE" -m "$big" feature-check --deep 2>/dev/null || true)
    printf '%s' "$out" | grep -qiF "skip" && pass "prints skip diagnostic above limit" || fail "no skip diagnostic above limit"
    "$SYSCRIBE" -m "$big" feature-check --deep >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -eq 0 ] && pass "skipped deep analysis exits 0 (no hang/false-OK)" || fail "guard exit ${ec}"
    rm -rf "$big"

    # Scenario: scope statement — Boolean layer only.
    SCENARIO_NAME="scope statement: Boolean layer only"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local scope; scope=$("$SYSCRIBE" -m "$A" feature-check --deep 2>/dev/null || true)
    printf '%s' "$scope" | grep -qiF "Boolean feature layer only" \
        && pass "output states Boolean-only scope" || fail "missing Boolean-only scope statement"
}
