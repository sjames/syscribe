// Bundles the extension for shipping: src/extension.ts (+ everything it
// imports, including vscode-languageclient) -> dist/extension.js. Only the
// `vscode` module stays external — it's provided by the Extension Host, not
// something we can or should bundle. Node built-ins (https, fs, path,
// child_process) are left as `require(...)` calls by esbuild's node
// platform, which is what we want (real Node, not a browser polyfill).
//
// Test files are *not* part of this bundle — they're compiled separately by
// plain `tsc` (see the `compile-tests` npm script) since @vscode/test-electron
// runs them as individual files, not as one bundle.
const esbuild = require("esbuild");

const production = process.argv.includes("--production");
const watch = process.argv.includes("--watch");

async function main() {
  const ctx = await esbuild.context({
    entryPoints: ["src/extension.ts"],
    bundle: true,
    format: "cjs",
    platform: "node",
    target: "node18",
    external: ["vscode"],
    outfile: "dist/extension.js",
    sourcemap: !production,
    minify: production,
    logLevel: "info",
  });

  if (watch) {
    await ctx.watch();
  } else {
    await ctx.rebuild();
    await ctx.dispose();
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
