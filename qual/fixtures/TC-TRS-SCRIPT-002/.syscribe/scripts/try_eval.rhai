// `eval` is disabled in the sandbox — calling it must error, not execute code.
fn do_eval(model) {
    let r = eval("1 + 1");
    "eval succeeded: " + r
}

register_command("try_eval", "Attempt eval (must be refused)", do_eval);
