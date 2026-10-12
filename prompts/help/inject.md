# inject — run a composite's safety invariants under injected faults

## SYNOPSIS
    syscribe inject <composite|TestCase> [--ft <FaultTree>] [--max-order <N>] [--fmea <FMEASheet>] [--report-only] [--config <C>] [--record [--run <id>]] [--report <file.json|file.md>] [--json]

## DESCRIPTION
The campaign of fault injection (see `syscribe help simulate` for the scenario steps). The scenarios of the
composite's TestCase that are tagged `@invariant` (a Gherkin tag on the line before `Scenario:`, or before
`Feature:` for all of them) state safety properties that must hold whatever fails. For every fault set the
faults are injected at time 0 and every invariant runs unchanged:

* every fault that applies to the composite — the `injection:` of an FMEA row or fault-tree event — is a set of its own
  (`--fmea <sheet>` restricts these to the rows of one sheet);
* with `--ft <FaultTree>`, every minimal cut set of that tree up to `--max-order` (default 2) whose events all have a
  fault model is a set; a cut set with an event that has none is listed as not run.

A set is **survived** when every invariant passes (**survived (degraded)** when one held only in an
`Except under` clause of a named property, see `syscribe help simulate`), **defeated** when one fails (the invariant and the failing step are
shown), and **error** when the run could not be made. The invariants must pass without faults first, else the campaign
stops: a property that is false without faults says nothing about the faults. The report also lists the FMEA rows of the
composite's parts that were exercised, those with a fault model that were not, and those with no fault model at all.

Faults are injected before the first step: a `Given` that sets a variable a fault holds (stuck) or a part that
failed silent has no effect, and an invariant must not contain `fault … is injected/cleared` steps. Common-cause
groups (`ccfGroup:`) run as the failure of every member together. Cut sets above `--max-order` are not run, and
every fault set rebuilds the system (and re-instantiates plants), so a large order over many faults takes time.

## OPTIONS
    --ft <FaultTree>      also run the minimal cut sets of this fault tree
    --max-order <N>       largest cut set to run (default 2)
    --fmea <FMEASheet>    only the single faults of this sheet's rows (fault-tree events and cut sets are unaffected)
    --report-only         report defeats and errors but exit 0
    --config <C>         run on the model projected onto this Configuration: feature parameters written as {{Features::F.param}} in guards, assignments and durations take its values
    --record              keep the campaign in `.syscribe/campaigns.json` (one history per subject and Configuration,
                          latest first); every defeat in the latest record then needs a `ResidualRisk` decision (W940)
    --run <id>            name the recorded run; a record with the same id is replaced (an unnamed one replaces the
                          earlier unnamed one)
    --report <file>       write the campaign with the decisions on each defeat and the failure rates of its faults,
                          as JSON, or Markdown for a `.md` file
    --json                machine-readable report

## RECORDED CAMPAIGNS (GH #341)
A recorded campaign is evidence: everything in it but the timestamp is a function of the model, so a second run on the
same model records the same content. `syscribe results campaigns` lists the histories, `results campaigns diff`
compares two records. `validate` checks the `ResidualRisk` decisions against the latest record of each campaign:
W940 a defeat no decision names, W941 a decision the campaign contradicts, W942 a decision taken on another composite
hash, W943 an expired decision, W944 a record of an older composite, W945 an unreadable record file (`--record`
then refuses to overwrite it), W946 a decision whose set the latest record did not run. Each record keeps its
invocation (`--ft`, `--max-order`, `--fmea`); re-recording with fewer sets warns and prints the earlier command.
`--run` without `--record` or `--report` is ignored with a warning. See spec §12.14.

## EXIT STATUS
0 every set survived (or `--report-only`) · 1 usage: no composite, no `@invariant` scenario, an unknown fault tree or
sheet · 2 a set defeated an invariant or could not be run, or an invariant fails without faults.
