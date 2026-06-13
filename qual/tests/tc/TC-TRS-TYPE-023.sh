tc_TRS_TYPE_023() {
    # Gitlink-vs-ref mismatch (W512). Builds a real parent repo with a git
    # submodule under a temp dir so the gitlink/ref comparison is deterministic.
    local out rc
    local GU="-c user.email=t@t.invalid -c user.name=qual"
    local GA="-c protocol.file.allow=always"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    _has() { printf '%s' "$1" | grep -qF "$2"; }

    local W; W=$(mktemp -d)

    # ── upstream peer repo: c1 @ v1, then c2 @ v2 ──
    mkdir -p "$W/subupstream/model"
    printf -- '---\ntype: Package\nname: PeerRoot\n---\n' > "$W/subupstream/model/_index.md"
    git -C "$W/subupstream" init -q
    git -C "$W/subupstream" add -A
    git -C "$W/subupstream" $GU commit -qm c1
    git -C "$W/subupstream" tag v1
    printf -- '---\ntype: PartDef\nname: Widget\ndomain: system\n---\n' > "$W/subupstream/model/Widget.md"
    git -C "$W/subupstream" add -A
    git -C "$W/subupstream" $GU commit -qm c2
    git -C "$W/subupstream" tag v2

    # ── parent repo with the composing model at parent/model + submodule @ HEAD=c2 ──
    mkdir -p "$W/parent/model"
    git -C "$W/parent" init -q
    printf -- '---\ntype: Package\nname: Integration\n---\n' > "$W/parent/model/_index.md"
    printf '[repos]\npeer = { path = "../vendor/peer", root = "model/", ref = "v1" }\n' > "$W/parent/model/.syscribe.toml"
    git -C "$W/parent" add -A
    git -C "$W/parent" $GU commit -qm init
    git -C "$W/parent" $GA $GU submodule add -q "$W/subupstream" vendor/peer >/dev/null 2>&1
    git -C "$W/parent" add -A
    git -C "$W/parent" $GU commit -qm "add submodule"

    # 1. ref v1 (c1) ≠ gitlink (c2) → W512
    _scn "W512 — ref disagrees with the submodule gitlink"
    out=$("$SYSCRIBE" -m "$W/parent/model" validate 2>&1) && rc=0 || rc=$?
    if _has "$out" 'W512' && _has "$out" 'peer' && _has "$out" 'gitlink'; then
        pass "W512 names the repo whose ref disagrees with the gitlink"
    else
        fail "W512 not emitted when ref disagrees with the submodule gitlink"
    fi

    # 2. --deny W512 gates CI
    _scn "--deny W512 gates CI"
    "$SYSCRIBE" -m "$W/parent/model" validate --deny W512 >/dev/null 2>&1 && rc=0 || rc=$?
    if [ "$rc" -ne 0 ]; then
        pass "validate --deny W512 exits non-zero ($rc) on a gitlink mismatch"
    else
        fail "validate --deny W512 did not gate on a gitlink mismatch (exit 0)"
    fi

    # 3. ref v2 (c2) == gitlink → no W512
    _scn "ref matching the gitlink is silent"
    sed -i 's/ref = "v1"/ref = "v2"/' "$W/parent/model/.syscribe.toml"
    out=$("$SYSCRIBE" -m "$W/parent/model" validate 2>&1) && rc=0 || rc=$?
    if _has "$out" 'W512'; then
        fail "W512 wrongly emitted when ref resolves to the gitlink commit"
    else
        pass "no W512 when the ref resolves to the recorded gitlink commit"
    fi

    # 4. a non-submodule (sibling) peer never emits W512
    _scn "non-submodule peer does not emit W512"
    mkdir -p "$W/sib/peer/model" "$W/sib/local"
    printf -- '---\ntype: Package\nname: PeerRoot\n---\n' > "$W/sib/peer/model/_index.md"
    git -C "$W/sib/peer" init -q
    git -C "$W/sib/peer" add -A
    git -C "$W/sib/peer" $GU commit -qm c1
    git -C "$W/sib/peer" tag v1
    printf '[repos]\npeer = { path = "../peer", root = "model/", ref = "v1" }\n' > "$W/sib/local/.syscribe.toml"
    printf -- '---\ntype: Package\nname: Integration\n---\n' > "$W/sib/local/_index.md"
    out=$("$SYSCRIBE" -m "$W/sib/local" validate 2>&1) && rc=0 || rc=$?
    if _has "$out" 'W512'; then
        fail "W512 wrongly emitted for a non-submodule sibling peer"
    else
        pass "no W512 for a non-submodule sibling peer"
    fi

    rm -rf "$W"
}
