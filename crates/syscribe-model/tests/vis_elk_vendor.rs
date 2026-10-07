//! `REQ-TRS-VIS-016`: the vendored `elk.bundled.js` the executable embeds
//! (`crates/syscribe-model/vendor/elkjs/`) is pinned to the `elkjs` version
//! the browser client depends on (`crates/syscribe-server/frontend/
//! package-lock.json`), so the two engines cannot drift apart silently.

use std::path::Path;

use syscribe_model::vis::layout::{elk_version, ELK_BUNDLE};

fn frontend() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../syscribe-server/frontend")
}

#[test]
fn the_vendored_bundle_version_equals_the_frontend_lockfile_pin() {
    let lock: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(frontend().join("package-lock.json")).expect("frontend/package-lock.json")).unwrap();
    let pinned = lock["packages"]["node_modules/elkjs"]["version"].as_str().expect("elkjs pinned in package-lock.json");
    assert_eq!(
        elk_version(),
        pinned,
        "vendor/elkjs/VERSION ({}) differs from the elkjs the client depends on ({pinned}): re-vendor lib/elk.bundled.js and update VERSION",
        elk_version()
    );
    assert!(!ELK_BUNDLE.is_empty() && ELK_BUNDLE.contains("ELK"), "the bundle is embedded");
}

#[test]
fn the_vendored_bundle_is_byte_identical_to_the_installed_one_when_present() {
    // `node_modules` is not committed; when it is installed, the vendored
    // copy must be exactly its `lib/elk.bundled.js`.
    let installed = frontend().join("node_modules/elkjs/lib/elk.bundled.js");
    let Ok(bytes) = std::fs::read(&installed) else { return };
    assert_eq!(bytes.len(), ELK_BUNDLE.len(), "{} differs in size from vendor/elkjs/elk.bundled.js", installed.display());
    assert!(bytes == ELK_BUNDLE.as_bytes(), "{} differs from vendor/elkjs/elk.bundled.js", installed.display());
}
