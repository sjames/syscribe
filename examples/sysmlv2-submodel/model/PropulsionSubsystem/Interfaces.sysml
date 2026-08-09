package Propulsion {

connection def PowerLink;

interface def PowerInterface {
    port supplyPort : PowerPort;
}

requirement def RotorThrustReqDef;

part def Drone {
    port powerPort : PowerPort;
    interface powerIface : PowerInterface;
    connection powerLink : PowerLink connect powerPort to rotorConfig;
    part rotorConfig : RotorConfigChoice;
    allocation motorAlloc : RotorAssembly;

    requirement thrustCheck : RotorThrustReqDef {
        verify 'REQ-DRONE-VERIFY-001';
    }

    satisfy Requirements::'REQ-DRONE-THRUST-001';
}

part droneInstance : Drone;

}
