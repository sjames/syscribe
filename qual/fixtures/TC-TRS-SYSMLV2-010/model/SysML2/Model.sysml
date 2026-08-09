package Demo {
    part def Ecu {
        port p1 : SomePort;
    }
    part def BinaryHolder {
        part a : Ecu;
        part b : Ecu;
        connection c : SomeConnDef connect a.p1 to b.p1;
    }
    part def NaryHolder {
        part a : Ecu;
        part b : Ecu;
        part c : Ecu;
        connection bus : SomeConnDef connect (a.p1, b.p1, c.p1);
    }
    part def NoConnectHolder {
        connection c : SomeConnDef;
    }
}
