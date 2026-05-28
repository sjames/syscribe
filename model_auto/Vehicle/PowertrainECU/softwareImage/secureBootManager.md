---
type: Part
name: Secure Boot Manager
typedBy: System::Software::SecureBootManager
domain: software
---

SecureBootManager instance on the PowertrainECU. Executes the three-stage
chain-of-trust at every reset: ROM loader verifies bootloader, bootloader
verifies application firmware, rollback counter enforced from OTP. Ensures
only OEM-signed, non-rolled-back firmware images are admitted to execution.
