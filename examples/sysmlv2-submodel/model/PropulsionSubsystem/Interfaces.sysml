package Propulsion {

connection def PowerLink;

interface def PowerInterface {
    doc /* @SyscribeShortName: power-if */
    port supplyPort : PowerPort;
}

requirement def RotorThrustReqDef;

part def Drone {
    port powerPort : PowerPort;
    interface powerIface : PowerInterface;
    connection powerLink : PowerLink connect powerPort to rotorConfig {
        doc /* Primary power feed from the airframe bus to the active rotor configuration. */
    }
    part rotorConfig : RotorConfigChoice;
    allocation motorAlloc : RotorAssembly;

    requirement thrustCheck : RotorThrustReqDef {
        verify 'REQ-DRONE-VERIFY-001';
    }

    satisfy Requirements::'REQ-DRONE-THRUST-001';
}

part droneInstance : Drone;

}
