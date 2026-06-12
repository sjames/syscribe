// Exercises the read-only model API: iteration, getters, find, custom_fields,
// e.field(), computed reverse indices, and print/eprint to stdout/stderr.

fn inspect(model) {
    eprint("inspect: starting");

    // Iterate requirements and read getters.
    for e in model.elements_of_type("Requirement") {
        print("REQ " + e.id + " status=" + e.status + " domain=" + e.field("reqDomain"));
    }

    // find by id and by qualified name resolve the same element.
    let by_id = model.find("REQ-FIX-BRK-001");
    let by_qname = model.find("Requirements::Brakes");
    print("find_id_qname=" + by_id.id + "/" + by_qname.id);

    // doc body, title, type, name, tags.
    print("title=" + by_id.title);
    print("type=" + by_id.type);
    print("tags=" + by_id.tags.len());
    print("doc_has_shall=" + by_id.doc.contains("shall"));

    // custom_fields map + e.field; absent field/custom field => unit ().
    let cf = by_id.custom_fields;
    print("supplier=" + cf["supplier"]);
    print("mg_origin=" + cf["mg_origin"]);
    let missing = by_id.field("nope");
    print("missing_is_unit=" + (missing == ()));

    // computed reverse index: verified_by.
    print("verified_by=" + by_id.verified_by.len());
    if by_id.verified_by.len() > 0 {
        print("verified_by_0=" + by_id.verified_by[0]);
    }

    // unknown find returns unit.
    let none = model.find("REQ-DOES-NOT-EXIST");
    print("unknown_is_unit=" + (none == ()));

    eprint("inspect: done");
    "inspect complete: " + model.elements().len() + " elements"
}

register_command("inspect", "Inspect the model via the read-only API", inspect);
