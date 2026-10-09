# fault-tree — fault tree analysis commands

## SYNOPSIS
    syscribe -m <root> fault-tree render <FaultTree-id>
    syscribe -m <root> fault-tree analyze <FaultTree-id> [--json] [--max-order N] [--no-ccf]

## DESCRIPTION
Sub-commands for FaultTree (IEC 61025 / ISO 26262-9) analysis.

`fault-tree render` emits a Mermaid `flowchart TD` string for the named FaultTree
element. The diagram shows all FaultTreeGate and FaultTreeEvent children with their
types and ids. Gate type (AND, OR, etc.) is shown in node labels; edges represent
the gate `inputs` list.

`fault-tree analyze` evaluates the gate logic of the named FaultTree and reports its
minimal cut sets (with order and probability), the exact top-event probability
(plus rare-event and min-cut-upper-bound approximations), and per-event importance
(Fussell-Vesely, Birnbaum, RAW) and role (single_point, dual_point, multi_point,
irrelevant, unreachable, house). An event's probability is its `probability:` or
1 - exp(-failureRate * missionTime). House events are constants (TRUE only at
probability 1). Events sharing `ccfGroup:` with `ccfBeta:` get a beta-factor
common-cause event (CCF:<group>). A tree with events but no gates is an implicit OR.
A gate cycle or a missing top node is an error (exit 1); see E960-E964, W960-W964.

## OPTIONS
    render <FaultTree-id>   Emit Mermaid flowchart for the named FaultTree.
    analyze <FaultTree-id>  Cut sets, top-event probability, importance, CCF.
      --json                Machine-readable document (cutSets, events, topProbability, ...).
      --max-order N         Discard cut sets of order > N (probabilities stay exact).
      --no-ccf              Ignore ccfGroup/ccfBeta (no common-cause expansion).

## EXAMPLES
    # against the bundled automotive model (model_auto/)
    syscribe -m model_auto/ fault-tree render FT-ENG-001
    syscribe -m model_auto/ fault-tree analyze FT-ENG-001
    syscribe -m model_auto/ fault-tree analyze FT-ENG-001 --json

## SEE ALSO
    fmea, metrics, validate, spec safety
