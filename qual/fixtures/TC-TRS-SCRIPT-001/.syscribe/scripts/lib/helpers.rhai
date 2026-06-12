// Reusable library module — registers nothing, only exports functions.
// Importable as `import "lib/helpers" as h;` from any script in the scripts dir.

fn banner(title) {
    "== " + title + " =="
}

fn count_reqs(model) {
    model.elements_of_type("Requirement").len()
}
