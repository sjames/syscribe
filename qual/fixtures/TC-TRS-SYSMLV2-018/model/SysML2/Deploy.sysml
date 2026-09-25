package Deploy {

    allocation def SoftwareToHardware {
        doc /* Deploys a software component onto a hardware board. */
    }

    part def Rack {
        allocation def RackSlot;
    }

    allocation deployCtl : SoftwareToHardware;
    allocation slotUse : Rack::RackSlot;
    allocation broken : NoSuchAllocationDef;
}
