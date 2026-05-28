---
type: Part
name: Secure Boot Manager
typedBy: System::Software::SecureBootManager
domain: software
---

SecureBootManager instance on the PowertrainECU. Executes the three-stage chain-of-trust
at every reset: ROM loader verifies bootloader, bootloader verifies application firmware,
rollback counter enforced from OTP. Only OEM-signed, non-rolled-back firmware images
are admitted to execution.

## Chain-of-trust stages

**Stage 1 — ROM loader (immutable):** The MCU's mask-ROM loader computes SHA-256 over the
primary bootloader flash region and verifies the ECDSA P-256 signature against the OEM root
CA certificate stored in OTP fuses. If verification fails, the MCU halts (no fall-through to
untrusted code); the ECU is non-functional until reflashed via JTAG.

**Stage 2 — Primary bootloader:** Verifies the application firmware partition signature using
the same ECDSA P-256 / SHA-256 scheme. Checks the monotonic version counter in OTP fuses:
if the firmware header version is lower than the stored counter, the image is rejected as a
rollback attempt. On success, increments the OTP counter (irreversible) and jumps to the
application entry point.

**Stage 3 — Application firmware:** The application records its version in non-volatile RAM
at first boot after a firmware update; this value is used by the DiagnosticSecurityLayer
audit log and by UDS ReadDataByIdentifier (DID 0xF189 — ECU software version number).

## Rollback attack mitigation

The OTP monotonic counter is the primary countermeasure for VR-ENG-002 (firmware rollback
via OBD-II). Until this counter is provisioned and enforced (`status: open` on VR-ENG-002),
the SecureBootManager accepts any validly-signed firmware regardless of version, leaving
previously-patched vulnerabilities exploitable via downgrade.

## Key-compromise recovery

If the OEM root CA private key is compromised, recovery requires: (1) fuse a new OEM root CA
public key hash into a secondary OTP slot, (2) issue a field update signed with the new key,
and (3) mark the old key slot as revoked. This procedure requires physical access to a JTAG
programmer and cannot be performed over-the-air via OBD-II.
