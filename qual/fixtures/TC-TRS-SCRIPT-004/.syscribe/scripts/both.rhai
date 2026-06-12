// A single file may register BOTH a command and a check.
fn report(model) {
    "reqs=" + model.elements_of_type("Requirement").len()
}

fn naming(model) {
    for e in model.elements_of_type("Requirement") {
        if e.status == "draft" {
            finding(e, "DRAFT", "info", "requirement is still draft");
        }
    }
}

register_command("report", "Report requirement count", report);
register_check("naming", "Flag draft requirements", naming);
