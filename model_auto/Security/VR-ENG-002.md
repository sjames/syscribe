---
type: VulnerabilityReport
id: VR-ENG-002
name: Firmware rollback attack via OBD-II reflash without rollback counter check
status: open
cvssScore: 6.8
attackVector: physical
mitigatedBy:
  - SC-ENG-003
---

## Summary

The Engine ECU bootloader accepts any firmware image that carries a valid ECDSA
digital signature issued by the OEM root CA. However, the bootloader does not
enforce a monotonic rollback counter: a signed older firmware image whose
signature has not been revoked remains cryptographically valid even after the
ECU fleet has been updated to a patched version.

An attacker with physical OBD-II access can therefore:

1. Obtain a legacy signed firmware binary (e.g., from an end-of-life ECU,
   a leaked OEM software archive, or a recycled part).
2. Initiate a UDS ECU programming session (service 0x34/0x36/0x37) after
   passing seed-and-key authentication.
3. Flash the older firmware image; the bootloader verifies the signature,
   finds it valid, and programs the device without checking the version number
   against any stored minimum.

## Impact

After a successful rollback, the ECU runs firmware that may pre-date the
SecOC initialisation fix described in `Security::VR-ENG-001`. This restores
the ~150 ms startup injection window during which unauthenticated CAN torque
commands are accepted, enabling physical-access attackers to combine both
vulnerabilities for a two-stage exploit without requiring key compromise.

Additional consequences include re-exposure of any other defects patched
in subsequent firmware releases, and potential violation of the OEM's type
approval obligations under UNECE Regulation No. 155, which requires that
software update mechanisms prevent the installation of software that
introduces new risks (R155 control 7.3.6).

## Attack Scenario

1. Attacker acquires a recycled Engine ECU from a second-hand vehicle.
2. Extracts the original firmware binary from flash using JTAG or OBD reflash.
3. On target vehicle: connects OBD-II scan tool, triggers extended diagnostic
   session, completes seed-and-key challenge for programming security level.
4. Transmits legacy firmware via UDS 0x36 (TransferData) blocks.
5. ECU bootloader verifies ECDSA signature — valid for legacy build — and
   programs flash.
6. ECU resets into old firmware; VR-ENG-001 startup window is now present.

## Mitigation

Security control `SC-ENG-003` (ECDSA P-256 signature verification with
monotonic OTP version counter) prevents this attack. The bootloader must
compare the firmware image version field against the value stored in OTP
memory and reject any image whose version is lower. Complies with UNECE R155
control 7.3.6 (protection against unauthorised software manipulations).

## References

- UNECE Regulation No. 155, control 7.3.6 — Software update security
- ISO/SAE 21434 §14 — Vulnerability management process
- Related threat scenario: `TS-ENG-003` (firmware rollback via OBD-II)
- Related cybersecurity goal: `CSG-ENG-003`
- Related vulnerability: `Security::VR-ENG-001` (SecOC startup window)
