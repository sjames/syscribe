// Dev-time bundling only (ADR-SYS-DE-001 consequence: "the project's first
// JS/TS build step"). The output is one static JS file served through
// syscribe-server's existing rust_embed pipeline (`static_assets.rs`), same as
// the vendored cytoscape/mermaid/htmx bundles under `static/js/` — there is no
// runtime Node/npm dependency, only this build step.
import { build } from 'esbuild';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const here = path.dirname(fileURLToPath(import.meta.url));
const outfile = path.resolve(here, '../static/js/diagram-editor.js');

await build({
  entryPoints: [path.join(here, 'src/main.ts')],
  bundle: true,
  outfile,
  format: 'iife',
  target: 'es2020',
  sourcemap: true,
  // sprotty's own views use the classic-JSX `/** @jsx svg */` pragma per file
  // (see `src/views.tsx`); this is the matching default when a file has no
  // pragma comment of its own.
  jsxFactory: 'svg',
  jsxFragment: 'Fragment',
  logLevel: 'info',
});

console.log(`Built ${outfile}`);
