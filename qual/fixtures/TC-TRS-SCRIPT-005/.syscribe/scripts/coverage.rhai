// A command and a check, used to exercise `scripts list` and `scripts run`.

fn coverage(model) {
    let reqs = model.elements_of_type("Requirement");
    "coverage: " + reqs.len() + " requirements"
}

fn naming(model) {
    for e in model.elements_of_type("Requirement") {
        if e.status == "draft" {
            finding(e, "DRAFT", "warning", "draft requirement");
        }
    }
}

register_command("coverage", "Summarise requirement coverage", coverage);
register_check("naming", "Flag draft requirements", naming);
