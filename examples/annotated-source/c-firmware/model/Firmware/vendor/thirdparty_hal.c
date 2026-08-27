// A vendored third-party HAL file. It happens to contain something that
// *looks* like a marker, but `Firmware/_index.md`'s `exclude: ["**/vendor/**"]`
// keeps this whole directory out of the scan — demonstrates `exclude:`
// winning over `include:`.

// @syscribe
// type: Part
// name: ShouldNeverAppear

void hal_init(void) {}
