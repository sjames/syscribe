# list — enumerate elements of a type

## SYNOPSIS
    syscribe -m <root> list <Type> [scope]
        [--tag <t>] [--feature <F>] [--metadata <D>] [--config <C>]
        [--status <s>] [--sil <v>] [--has-wcet] [--open-questions] [--rate-status <s>]
        [--where custom.<key>[<op><value>]]... [--json]

## DESCRIPTION
Lists elements of a given type, optionally scoped to a namespace, with filters
for tags, feature gating, lifecycle status, integrity level, and WCET claims.

<Type> is any element type's canonical name — exactly the `type:` value an
author writes (Requirement, PartDef, Zone, Conduit, TestPlan, …), matched
case-insensitively. Every type in the inventory is accepted (`template` with an
unknown type prints the full list).

## OPTIONS
    [scope]          Namespace prefix to restrict to (e.g. UAV::Avionics).
    --tag <t>        Keep only elements whose tags: include t. Repeatable; repeated tags are ANDed
                     (the element must carry every one).
    --feature <F>    Keep only elements gated by FeatureDef F (via appliesWhen).
    --metadata <D>   Keep only elements that apply the stereotype MetadataDef D
                     (a `metadata:` application; matched by qname or last segment).
                     Repeatable: every one must hold. `A,B` accepts either; a leading `!`
                     negates (`--metadata '!Optional'`). A name that is no MetadataDef of the
                     model and that no element applies is a usage error (exit 1).
    --config <C>     Project onto a configuration (only active elements).
    --status <s>     Keep only elements whose status: equals s.
    --sil <v>        Keep only elements whose silLevel stringifies to v OR asilLevel == v.
    --has-wcet       Keep only elements that declare a non-empty wcet:.
    --open-questions Keep only PlanningItems with at least one open `questions:` entry (see `questions`);
                     the --json form of PlanningItem carries an `openQuestions` count.
    --rate-status <s> Keep only the fault-tree events and FMEA rows whose rateStatus: is s
                     (verified, secondary, unverified; `none` for a rate with no status). Only
                     FaultTreeEvent and FMEA rows are filtered; on any other type it is a
                     silent no-op.
    --where custom.<key>[<op><value>]   Filter by custom_fields:. <op> is `=` (exact; any
                     list element), `=~` (regex, substring fallback) or `~=` (list
                     membership); a bare custom.<key> tests presence. Any other operator
                     (!=, ==, ~, >, <, >=, <=) is a usage error (exit 1). Repeatable, ANDed.
    --where meta.<Def>.<key><op><value>   Filter by a stereotype's tagged value, defaults and
                     all. <op> is `=`, `!=`, `=~`, `~=`, or `<` `<=` `>` `>=` against a number;
                     `meta.<Def>.<key>` tests that a value is present and `meta.<Def>` that the
                     stereotype is applied. An element without the stereotype or value never
                     matches (`!=` included). Also accepted by `ls`, `find`, `digest`, `stats`.
    --json           Emit a JSON array (qualifiedName,type,name,id,status,silLevel,asilLevel,wcet);
                     an element that applies stereotypes also carries `metadata`:
                     [{def, values, defaulted}] with declared defaults filled in.

## EXAMPLES
    syscribe -m model/ list Requirement
    syscribe -m model/ list PartDef --where 'meta.DALAnnotation.level<=2'
    syscribe -m model/ list PartDef --metadata DALAnnotation --metadata '!ModelStatus'
    syscribe -m model/ list PartDef UAV::Avionics
    syscribe -m model_sil/ list Requirement --status approved --sil 4
    syscribe -m <root> list Requirement --has-wcet --json   # [] unless the model sets wcet: (no bundled model does)
    syscribe -m model/ list Requirement --tag config --tag plantuml --json
    syscribe -m model_mg/ list PartDef --where custom.mg_layer=physical

## SEE ALSO
    types, find, matrix, verification-depth
