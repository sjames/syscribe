package Demo {
    part def Ecu;
    part def Holder {
        part a : Ecu;
        part b : Ecu;
        connection withDoc : SomeConnDef connect a to b {
            doc /* Explanation. */
        }
        connection noBody : SomeConnDef connect a to b;
    }
}
