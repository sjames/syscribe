// A well-formed command that must still load and run deterministically even
// though sibling scripts attempt forbidden things or fail to parse.
fn ok(model) {
    "ok:" + model.elements_of_type("Requirement").len()
}

register_command("ok", "Healthy command unaffected by sibling failures", ok);
