# fault-tree — fault tree analysis commands

## SYNOPSIS
    syscribe -m <root> fault-tree render <FaultTree-id>

## DESCRIPTION
Sub-commands for FaultTree (IEC 61025 / ISO 26262-9) analysis.

`fault-tree render` emits a Mermaid `flowchart TD` string for the named FaultTree
element. The diagram shows all FaultTreeGate and FaultTreeEvent children with their
types and ids. Gate type (AND, OR, etc.) is shown in node labels; edges represent
the gate `inputs` list.

## OPTIONS
    render <FaultTree-id>   Emit Mermaid flowchart for the named FaultTree.

## EXAMPLES
    # against the bundled automotive model (model_auto/)
    syscribe -m model_auto/ fault-tree render FT-ENG-001

## SEE ALSO
    fmea, validate, spec safety
