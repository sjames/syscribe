tc_TRS_MOVE_001() {
    local F="$1"; local BASE="$F/move/base"
    local W rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    _ok()  { [ "$1" = "1" ] && pass "$2" || fail "$2"; }

    # Move a single element into a new namespace.
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    "$SYSCRIBE" -m "$W" move Pkg::Sub::Widget Pkg::Other::Widget >/dev/null 2>&1 || true
    _scn "move single element"
    [ -f "$W/Pkg/Other/Widget.md" ] && [ ! -f "$W/Pkg/Sub/Widget.md" ] && _ok 1 "file relocated" || _ok 0 "file relocated"
    rm -rf "$W"

    # Move a package — the whole subtree relocates.
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    "$SYSCRIBE" -m "$W" move Pkg::Sub Pkg::Moved >/dev/null 2>&1 || true
    _scn "move package subtree"
    { [ -f "$W/Pkg/Moved/Widget.md" ] && [ -f "$W/Pkg/Moved/WidgetExtended.md" ] && [ ! -d "$W/Pkg/Sub" ]; } && _ok 1 "subtree relocated" || _ok 0 "subtree relocated"
    rm -rf "$W"

    # Moving onto an existing destination is rejected; nothing changes.
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    "$SYSCRIBE" -m "$W" move Pkg::A Pkg::B >/dev/null 2>&1 && rc=0 || rc=$?
    _scn "reject move onto existing destination"
    { [ "$rc" -ne 0 ] && [ -f "$W/Pkg/A.md" ] && [ -f "$W/Pkg/B.md" ]; } && _ok 1 "rejected, A and B intact" || _ok 0 "rejected, A and B intact"
    rm -rf "$W"

    # Moving a package into its own subtree is rejected.
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    "$SYSCRIBE" -m "$W" move Pkg::Sub Pkg::Sub::Inner >/dev/null 2>&1 && rc=0 || rc=$?
    _scn "reject move into own subtree"
    { [ "$rc" -ne 0 ] && [ -d "$W/Pkg/Sub" ]; } && _ok 1 "rejected, Sub intact" || _ok 0 "rejected, Sub intact"
    rm -rf "$W"
}
