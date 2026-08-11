package Demo {
    interface def IFoo;
    part def A;
    part def B;
    part def Ecu {
        port p1 : SomePort;
    }
    part def Resolved {
        part a : A {
            interface fooProvider : IFoo;
        }
        part b : B {
            interface fooClient : IFoo;
        }
        connection link1 : Link connect a.fooProvider to b.fooClient;
    }
    part def Inherited {
        part a : Ecu;
        part b : Ecu;
        connection link2 : Link connect a.p1 to b.p1;
    }
    part def Bare {
        part a : Ecu;
        part b : Ecu;
        connection link3 : Link connect a to b;
    }
    part def ThreeSegment {
        part a : A {
            interface fooProvider : IFoo;
        }
        part b : B;
        connection link4 : Link connect a.fooProvider.deep to b;
    }
}
