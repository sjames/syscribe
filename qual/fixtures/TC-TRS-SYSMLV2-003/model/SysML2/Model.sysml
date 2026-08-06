package Demo {
    part def Widget {
        satisfy 'REQ-DEMO-ONE-001';
    }
    part def Gadget {
        satisfy Requirements::'REQ-DEMO-TWO-001';
    }
    requirement checkThing {
        verify 'REQ-DEMO-THREE-001';
    }
}
