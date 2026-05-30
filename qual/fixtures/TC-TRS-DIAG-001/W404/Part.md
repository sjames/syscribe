---
type: PartDef
name: MyPart
operations:
  - name: doSomething
    parameters:
      - name: input
        typedBy: NonExistentType
    returnType: NonExistentReturnType
---

PartDef with operations whose parameter typedBy and returnType reference non-existent elements — should produce W404.
