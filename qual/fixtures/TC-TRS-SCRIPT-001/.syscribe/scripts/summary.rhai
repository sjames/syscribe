// Command script that reuses the shared library module.
import "lib/helpers" as h;

fn summary(model) {
    let n = h::count_reqs(model);
    h::banner("Requirement summary") + "\n" + "requirements: " + n
}

register_command("summary", "Print a one-line requirement summary", summary);
