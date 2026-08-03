// Toy SysMLv2-textual-subset parser — Syscribe WASM foreign-format plugin
// example (ADR-SYS-PLUGIN-001, REQ-TRS-PLUGIN-*).
//
// Recognises a tiny fragment of SysMLv2 textual notation:
//
//   part def <Name> {
//     doc "..."
//   }
//
//   requirement def <Name> {
//     id "REQ-..."
//     doc "..."
//   }
//
// This is intentionally not a real SysMLv2 grammar — it exists to prove the
// host<->plugin pipeline end to end (scoped file access, envelope shape,
// merge into the Syscribe graph, cross-reference resolution, diagnostics).

interface EnvelopeElement {
  qname: string;
  type: string;
  id?: string;
  name?: string;
  doc: string;
}

interface EnvelopeDiagnostic {
  severity: "error" | "warning";
  message: string;
  source_ref?: string;
}

interface ElementsEnvelope {
  elements: EnvelopeElement[];
  diagnostics: EnvelopeDiagnostic[];
}

const TYPE_MAP: Record<string, string> = {
  part: "PartDef",
  requirement: "RequirementDef",
};

const BLOCK_RE = /(part|requirement)\s+def\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{([^}]*)\}/g;
const DOC_RE = /doc\s+"([^"]*)"/;
const ID_RE = /\bid\s+"([^"]*)"/;

function parseFile(fileName: string, text: string, out: ElementsEnvelope): void {
  BLOCK_RE.lastIndex = 0;
  let match: RegExpExecArray | null;
  let found = false;
  while ((match = BLOCK_RE.exec(text)) !== null) {
    found = true;
    const [, kind, name, body] = match;
    const docMatch = DOC_RE.exec(body);
    const idMatch = ID_RE.exec(body);
    out.elements.push({
      qname: name,
      type: TYPE_MAP[kind],
      id: idMatch ? idMatch[1] : undefined,
      name,
      doc: docMatch ? docMatch[1] : "",
    });
  }
  if (!found) {
    out.diagnostics.push({
      severity: "warning",
      message: "no 'part def'/'requirement def' blocks recognised in this file",
      source_ref: fileName,
    });
  }
}

function parse(): void {
  const { fs_list_dir, fs_read } = Host.getFunctions();
  const out: ElementsEnvelope = { elements: [], diagnostics: [] };

  const listOffset = fs_list_dir(Memory.fromString(".").offset);
  let names: string[];
  try {
    names = JSON.parse(Memory.find(listOffset).readString());
  } catch (e) {
    Host.outputString(JSON.stringify(out));
    return;
  }

  for (const name of names) {
    if (!name.endsWith(".sysml")) continue;
    try {
      const readOffset = fs_read(Memory.fromString(name).offset);
      const text = new TextDecoder().decode(Memory.find(readOffset).readBytes());
      parseFile(name, text, out);
    } catch (e) {
      out.diagnostics.push({
        severity: "error",
        message: `failed to read/parse '${name}': ${e}`,
        source_ref: name,
      });
    }
  }

  Host.outputString(JSON.stringify(out));
}

module.exports = { parse };
