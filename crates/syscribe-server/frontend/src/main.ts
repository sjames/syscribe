// Entry point — bundled by esbuild into `static/js/diagram-editor.js` and
// loaded as a plain `<script>` in `templates/base.html`, exactly like the
// other vendored bundles under `static/js/` (`ADR-SYS-DE-001` consequence:
// "the served artifact remains a plain static file through the existing
// rust_embed pipeline"). Exposes a single `window.DiagramEditor` instance
// that `base.html`'s existing tab-management JS calls into for any
// non-`Mermaid`-kind diagram tab.
import 'reflect-metadata';
import { DiagramEditor } from './editor';

declare global {
    interface Window {
        DiagramEditor: DiagramEditor;
    }
}

window.DiagramEditor = new DiagramEditor();
