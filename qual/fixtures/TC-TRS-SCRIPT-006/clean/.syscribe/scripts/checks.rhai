// Validation-hook checks for `scripts validate`. Emits findings of differing
// severities so the gate/exit-code contract can be exercised.

fn naming(model) {
    for e in model.elements_of_type("Requirement") {
        if e.status == "draft" {
            // Warning-severity finding, namespaced as naming/DRAFT.
            finding(e, "DRAFT", "warning", "requirement is still draft");
        }
    }
}

fn supplier(model) {
    for e in model.elements_of_type("Requirement") {
        let cf = e.custom_fields;
        if cf["supplier"] == () {
            finding(e, "NOSUP", "error", "requirement missing supplier");
        }
    }
}

register_check("naming", "Flag draft requirements (warning)", naming);
register_check("supplier", "Require a supplier custom field (error)", supplier);
