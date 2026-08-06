package Demo {
    variation part def RotorChoice {
        variant part quadVariant : QuadKind {
            @SyscribeFeature {
                featureId = 'FEAT-ROTOR-QUAD';
            }
        }
        variant part hexVariant : HexKind {
            @SyscribeFeature {
                featureId = 'FEAT-ROTOR-HEX';
            }
        }
        variant part plainVariant : PlainKind;
    }
    part rotorUsage : RotorChoice;
}
