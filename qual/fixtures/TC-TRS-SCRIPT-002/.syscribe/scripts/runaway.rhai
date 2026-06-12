// Command whose function never terminates on its own — the engine's operation
// budget must abort it with an error (bounded time, non-zero exit), not hang.
fn spin(model) {
    let i = 0;
    loop {
        i += 1;
    }
    "unreachable"
}

register_command("runaway", "Infinite loop — must be aborted by the op limit", spin);
