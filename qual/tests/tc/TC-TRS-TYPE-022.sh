tc_TRS_TYPE_022() {
    # Ref-drift detection (W511). Builds real, isolated git work trees under a
    # temp dir so HEAD/ref comparisons are deterministic and unaffected by the
    # syscribe repo's own .git.
    local out rc
    local G="-c user.email=t@t.invalid -c user.name=qual"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    _has() { printf '%s' "$1" | grep -qF "$2"; }

    local W; W=$(mktemp -d)

    # ── peer git repo: commit c1 @ tag v1, then c2 so HEAD moves past v1 ──
    mkdir -p "$W/peer/model"
    printf -- '---\ntype: Package\nname: PeerRoot\n---\n' > "$W/peer/model/_index.md"
    git -C "$W/peer" init -q
    git -C "$W/peer" add -A
    git -C "$W/peer" $G commit -qm c1
    git -C "$W/peer" tag v1
    printf -- '---\ntype: PartDef\nname: Widget\ndomain: system\n---\n' > "$W/peer/model/Widget.md"
    git -C "$W/peer" add -A
    git -C "$W/peer" $G commit -qm c2

    # ── local model pinned to v1 (now drifted from HEAD=c2) ──
    mkdir -p "$W/local"
    printf '[repos]\npeer = { path = "../peer", root = "model/", ref = "v1" }\n' > "$W/local/.syscribe.toml"
    printf -- '---\ntype: Package\nname: Integration\n---\n' > "$W/local/_index.md"

    # 1. W511 fires on drift
    _scn "W511 — peer HEAD drifted from its pinned ref"
    out=$("$SYSCRIBE" -m "$W/local" validate 2>&1) && rc=0 || rc=$?
    if _has "$out" 'W511' && _has "$out" "peer" && _has "$out" "v1"; then
        pass "W511 names the drifted repo and its configured ref"
    else
        fail "W511 not emitted for a drifted peer"
    fi

    # 2. --deny W511 gates CI (non-zero exit)
    _scn "--deny W511 gates CI"
    "$SYSCRIBE" -m "$W/local" validate --deny W511 >/dev/null 2>&1 && rc=0 || rc=$?
    if [ "$rc" -ne 0 ]; then
        pass "validate --deny W511 exits non-zero ($rc) on drift"
    else
        fail "validate --deny W511 did not gate on drift (exit 0)"
    fi

    # 3. in-sync peer is silent
    _scn "in-sync peer is silent"
    git -C "$W/peer" checkout -q v1
    out=$("$SYSCRIBE" -m "$W/local" validate 2>&1) && rc=0 || rc=$?
    if _has "$out" 'W511'; then
        fail "W511 wrongly emitted when the peer is at its pinned ref"
    else
        pass "no W511 when the peer HEAD is at the configured ref"
    fi

    # 4. a non-git peer pinned to a ref does not warn (drift undeterminable)
    _scn "undeterminable drift does not warn"
    mkdir -p "$W/nogit/peer/model" "$W/nogit/local"
    printf -- '---\ntype: Package\nname: PeerRoot\n---\n' > "$W/nogit/peer/model/_index.md"
    printf '[repos]\npeer = { path = "../peer", root = "model/", ref = "v1" }\n' > "$W/nogit/local/.syscribe.toml"
    printf -- '---\ntype: Package\nname: Integration\n---\n' > "$W/nogit/local/_index.md"
    out=$("$SYSCRIBE" -m "$W/nogit/local" validate 2>&1) && rc=0 || rc=$?
    if _has "$out" 'W511'; then
        fail "W511 wrongly emitted for a non-git peer (drift cannot be determined)"
    else
        pass "no W511 when drift cannot be determined (non-git peer)"
    fi

    rm -rf "$W"
}
