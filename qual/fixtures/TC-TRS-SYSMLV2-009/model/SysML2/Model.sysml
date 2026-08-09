package Demo {
    part def SingleDocPart {
        doc /* Explanation. */
    }
    part def TwoDocPart {
        doc /* First. */
        doc /* Second. */
    }
    part def PlainPart;

    port def DocPortDef {
        doc /* port def doc. */
    }
    interface def DocIfaceDef;
    part def Holder {
        interface myIface : DocIfaceDef {
            doc /* interface usage doc. */
        }
    }
}
