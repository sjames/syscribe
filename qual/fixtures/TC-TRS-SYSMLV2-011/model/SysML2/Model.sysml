package Demo {
    part def Ecu;
    part def Holder {
        part a : Ecu;
        part b : Ecu;
        connection c : SomeConnDef connect a to b;
    }
}
