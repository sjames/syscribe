package Demo {
    interface def IPowerInterface {
        doc /*
        Real interface documentation.
        @SyscribeShortName: power-if
        @SyscribeImplementedBy: aidl/interfaces/car/power/IPowerInterface.aidl
        More prose after the directives.
        */
    }
    port def PowerPort {
        doc /* @SyscribeDomain: hardware
        @SyscribeIntegrity: asil=D */
    }
    connection def PowerLink {
        doc /* @SyscribeShortName: power-link */
    }
    interface def IPlain;
}
