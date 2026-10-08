---
id: REQ-TRS-FMED-003
type: Requirement
name: A configurator propagates selections, refuses or explains conflicts, shows the number of valid products, and saves a Configuration
status: verified
reqDomain: software
verificationMethod: test
---

`POST /api/feature-model/configure` **shall** take a partial selection and return, for every feature,
whether it is selected, deselected, forced on, forced off or free; whether the selection can still be
completed and, if not, the smallest set of the user's choices that clash and the constraints they
clash with; the number of valid products that remain, or a lower bound when counting reaches its
budget; and one complete product consistent with the choices. The page **shall** let a user select,
deselect and clear features, show the propagated state on the diagram, refuse a choice that no valid
product satisfies while explaining it, load a stored `Configuration` as the starting point, and save
the completed product as a `Configuration` through a guarded write.

**Source:** `REQ-TRS-FMED-003` (product model).

**Acceptance criteria:** (a) a choice forces what it requires and forbids the rest of an alternative
group; (b) an empty selection leaves mandatory features forced on and optional ones free, with the
product count; (c) a conflicting selection names only the clashing choices and the constraints, not
innocent choices; (d) unknown features are reported and ignored; (e) stored configurations are listed
in canonical form; (f) a saved completion is a `Configuration` the analysis calls valid; (g) a click
cycles undecided, selected, deselected, undecided, and the state is applied to the diagram by
qualified name.
