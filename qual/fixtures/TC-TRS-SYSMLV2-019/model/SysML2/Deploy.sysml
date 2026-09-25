package Deploy {

    allocation def SoftwareToHardware;

    part def CtlSw;
    part def Mcu;
    part def Controller {
        part ctl : CtlSw;
    }
    part def Hw {
        part mcu : Mcu;
    }

    part sys : Controller;
    part board : Hw;

    // Cross-package endpoints, resolved from Deploy's scope outward to the model root.
    allocation deploy : SoftwareToHardware allocate Arch::SwPackage to Arch::Board;

    // Feature chains resolved through the heads' part-def types (inherited features).
    allocation chained allocate sys.ctl to board.mcu;

    // A chain tail that exists nowhere: truncated to the head with W542.
    allocation truncated allocate sys.nosuch to board;

    // Endpoints that resolve nowhere: E502 / E503.
    allocation dangling allocate NoSuchSource to NoSuchTarget;
}
