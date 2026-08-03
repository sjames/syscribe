// Test-only fixture: never returns. Proves the host's timeout_ms/fuel budget
// actually interrupts a hung plugin rather than hanging validate() forever
// (ADR-SYS-PLUGIN-001, REQ-TRS-PLUGIN-003).
function parse(): void {
  let x = 0;
  while (true) {
    x = (x + 1) % 2147483647;
  }
}

module.exports = { parse };
