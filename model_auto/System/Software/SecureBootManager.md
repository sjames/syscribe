---
type: PartDef
name: Secure Boot Manager
domain: software
satisfies:
  - REQ-ENG-SEC-003
features:
  - name: otpVersionCounter
    type: ScalarValues::Integer
  - name: rootCACertificate
    type: ScalarValues::String
  - name: signatureAlgorithm
    type: ScalarValues::String
  - name: hashAlgorithm
    type: ScalarValues::String
---

Software component implementing the chain-of-trust boot sequence for the
Engine ECU, enforcing firmware integrity and rollback prevention per
`Requirements::Security::REQ-ENG-SEC-003`.

## Chain of Trust

The secure boot chain has three stages:

1. **ROM Loader (immutable)** — Executes from on-chip ROM at reset vector.
   Verifies the cryptographic signature of the primary bootloader using the
   OEM root CA public key burned into OTP memory at manufacturing time.
   Cannot be updated or overwritten by any software path.

2. **Primary Bootloader** — Verified by the ROM loader. Verifies the ECDSA
   P-256 signature (over the SHA-256 image hash) of the application firmware
   before transferring control. Reads the 32-bit monotonic version counter
   from the adjacent OTP word and rejects any firmware image whose embedded
   version field is less than or equal to the stored counter value.

3. **Application Firmware** — Verified by the primary bootloader. On first
   successful boot following a version-advancing update, the bootloader
   increments the OTP counter to match the new firmware version.

## Key Storage

The OEM root CA certificate (X.509, RSA-4096 or ECDSA P-384 outer key) is
stored in OTP memory during ECU manufacturing. The corresponding private key
is held exclusively within the OEM's HSM (Hardware Security Module) and never
leaves that boundary.

A separate OEM Recovery CA certificate is stored in a second OTP region to
support counter reset under physical OEM custody only, following key
compromise.

## Rollback Prevention

The OTP version counter is implemented as a fuse-based monotonic counter:
each increment burns additional fuse bits and is irreversible. Software cannot
decrement or reset the counter. Any firmware image with a version number lower
than the burned counter value is rejected with UDS NRC 0x22, and a security
DTC is set in non-volatile DTC memory.
