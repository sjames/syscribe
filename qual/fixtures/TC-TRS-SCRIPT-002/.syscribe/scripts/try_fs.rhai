// Attempt to escape the scripts directory via a relative import. The module
// resolver is confined to the scripts dir, so this must fail rather than read
// a file outside it. (No direct filesystem API exists in the sandbox at all.)
fn escape(model) {
    import "../../../../../etc/passwd" as p;
    "escaped"
}

register_command("try_fs", "Attempt to read outside the scripts dir (must fail)", escape);
