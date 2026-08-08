package Propulsion {

item def Fuel;

port def PowerPort;
port def FuelPort;

attribute def ThrustRating;

part def RotorAssembly {
    port fuelSupplyPort : FuelPort;
    item fuelItem : Fuel;
    attribute thrustReading : ThrustRating;

    @SyscribeDomain {
        value = 'hardware';
    }
    @SyscribeIntegrity {
        asil = 'B';
    }
    @SyscribeShortName {
        value = 'rotor-assembly';
    }

    satisfy 'REQ-DRONE-ENDUR-001';
}

variation part def RotorConfigChoice {
    variant part quadConfig : RotorAssembly {
        @SyscribeFeature {
            featureId = 'FEAT-ROTOR-QUAD';
        }
    }
    variant part hexConfig : RotorAssembly {
        @SyscribeFeature {
            featureId = 'FEAT-ROTOR-HEX';
        }
    }
}

}
