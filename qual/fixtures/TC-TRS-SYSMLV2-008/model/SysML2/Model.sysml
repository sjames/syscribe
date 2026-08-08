package Demo {
    part def AllFieldsPart {
        @SyscribeDomain {
            value = 'software';
        }
        @SyscribeIntegrity {
            asil = 'B';
        }
        @SyscribeShortName {
            value = 'all-fields-part';
        }
        @SyscribeImplementedBy {
            path = 'services/all-fields-part/';
        }
    }
    part def BothIntegrityPart {
        @SyscribeIntegrity {
            asil = 'D';
            sil = 2;
        }
    }
    part def PlainPart;
}
