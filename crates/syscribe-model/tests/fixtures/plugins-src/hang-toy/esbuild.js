const esbuild = require("esbuild");

esbuild.build({
  entryPoints: ["src/index.ts"],
  outdir: "dist",
  bundle: true,
  sourcemap: false,
  minify: false,
  format: "cjs",
  target: ["es2020"],
}).catch(() => process.exit(1));
