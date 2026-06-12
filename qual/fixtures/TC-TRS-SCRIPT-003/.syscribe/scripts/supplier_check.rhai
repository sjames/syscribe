// A check that emits findings via finding(elem, code, severity, msg).
fn supplier(model) {
    for e in model.elements_of_type("Requirement") {
        let cf = e.custom_fields;
        if cf["supplier"] == () {
            finding(e, "NOSUP", "warning", "requirement has no supplier custom field");
        }
    }
}

register_check("supplier", "Flag requirements missing a supplier", supplier);
