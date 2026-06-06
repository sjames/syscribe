tc_TRS_MOVE_003() {
    local F="$1"; local BASE="$F/move/base"
    local W rc before after

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    _sum() { find "$1" -type f -exec md5sum {} \; | sed "s#$1##" | sort; }

    # A rejected move (destination exists) must leave every file unchanged.
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    before=$(_sum "$W")
    "$SYSCRIBE" -m "$W" move Pkg::A Pkg::B >/dev/null 2>&1 && rc=0 || rc=$?
    after=$(_sum "$W")
    _scn "rejected move leaves model byte-for-byte unchanged"
    { [ "$rc" -ne 0 ] && [ "$before" = "$after" ]; } && pass "non-zero exit, no file changed" || fail "model changed or exit 0"
    rm -rf "$W"

    # --dry-run never writes.
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    before=$(_sum "$W")
    "$SYSCRIBE" -m "$W" move Pkg::Sub::Widget Pkg::Other::Widget --dry-run >/dev/null 2>&1 && rc=0 || rc=$?
    after=$(_sum "$W")
    _scn "--dry-run reports without writing"
    { [ "$rc" -eq 0 ] && [ "$before" = "$after" ]; } && pass "dry-run made no changes" || fail "dry-run modified files or failed"
    rm -rf "$W"
}
