// Wasm interface declaration consumed by `extism-js -i` (ADR-SYS-PLUGIN-001).
// Distinct from the ambient `@extism/js-pdk` editor types in tsconfig.json.

declare module "main" {
  export function parse(): I32;
}

declare module "extism:host" {
  interface user {
    // Scoped filesystem RPC (REQ-TRS-PLUGIN-003) — read-only, canonicalize +
    // prefix-checked against the plugin's declared package subtree on the host
    // side. `path` is always relative to that subtree.
    fs_read(ptr: I64): I64;
    fs_list_dir(ptr: I64): I64;
    fs_exists(ptr: I64): I64;
  }
}
