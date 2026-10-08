// Pure form logic of the "New diagram" dialog (`REQ-TRS-VIS-023`): the kinds
// that have a generator and the subject types each accepts (spec §8.16.8),
// name validation, the default package, and the `POST /api/elements` request
// built from the form. No DOM, no fetch — `new-diagram-dialog.ts` wires this
// to the page, and `test/new-diagram.test.mjs` drives it directly.

/** The diagram kinds a user can start: every kind with a generator
 * (`REQ-TRS-VIS-004`/`-005`/`-018`..`-022`). `subjectTypes` are the element
 * types spec §8.16.8 allows as a subject, which are also the suggestions the
 * dialog offers (the same set the validator's `W418` accepts). */
export interface DiagramKindInfo {
    kind: string;
    label: string;
    subjectTypes: string[];
    /** What a subject of this kind is, for the dialog's hint line. */
    hint: string;
}

export const DIAGRAM_KINDS: DiagramKindInfo[] = [
    { kind: 'BDD', label: 'Block definition (BDD)', subjectTypes: ['Package', 'PartDef', 'ItemDef'], hint: 'a package or a part/item definition' },
    { kind: 'IBD', label: 'Internal block (IBD)', subjectTypes: ['PartDef', 'Part', 'ItemDef', 'Item'], hint: 'a part or item, definition or usage' },
    { kind: 'StateMachine', label: 'State machine', subjectTypes: ['StateDef', 'State', 'ExhibitState'], hint: 'a state definition or state' },
    { kind: 'Action', label: 'Action flow', subjectTypes: ['ActionDef', 'Action'], hint: 'an action definition or action' },
    { kind: 'Sequence', label: 'Sequence', subjectTypes: ['ActionDef', 'Action', 'UseCaseDef', 'UseCase'], hint: 'an action or use case with send/accept steps' },
    { kind: 'Requirement', label: 'Requirement tree', subjectTypes: ['Package', 'RequirementDef', 'Requirement'], hint: 'a package of requirements, or one requirement' },
    { kind: 'FeatureModel', label: 'Feature model', subjectTypes: ['Package', 'FeatureDef', 'FeatureModel'], hint: 'a package of features, a feature, or a feature-model sheet' },
    { kind: 'Allocation', label: 'Allocation map', subjectTypes: ['Package', 'AllocationDef', 'Allocation'], hint: 'a package of allocations, or one allocation' },
];

export function kindInfo(kind: string): DiagramKindInfo | undefined {
    return DIAGRAM_KINDS.find(k => k.kind === kind);
}

/** The basic-name grammar every name-identified element follows. */
const BASIC_NAME = /^[A-Za-z_][A-Za-z0-9_]*$/;

/** `null` when `name` is acceptable, else the reason. */
export function validateName(name: string): string | null {
    const n = name.trim();
    if (n === '') {
        return 'Give the diagram a name.';
    }
    if (!BASIC_NAME.test(n)) {
        return 'A name uses letters, digits and underscores only, and does not start with a digit (no spaces or hyphens).';
    }
    return null;
}

/** The package the dialog preselects: a package named exactly `Diagrams` when
 * the model has one, else the model root (the empty qualified name). */
export function defaultPackage(packages: string[]): string {
    return packages.includes('Diagrams') ? 'Diagrams' : '';
}

/** `pkg::name`, or just `name` for the model root. */
export function joinQname(pkg: string, name: string): string {
    return pkg === '' ? name : `${pkg}::${name}`;
}

/** The tab id the page uses for a diagram: its qualified name with `/` for `::`
 * (the model browser's `url_path`). */
export function tabId(qname: string): string {
    return qname.replace(/::/g, '/');
}

export type StartFrom = 'derive' | 'blank';

export interface NewDiagramForm {
    name: string;
    kind: string;
    startFrom: StartFrom;
    subject: string;
    /** Qualified name of the package, `''` for the model root. */
    pkg: string;
    /** Qualified names the subject may be: the elements of the types
     * `kindInfo(kind).subjectTypes`, as fetched by the dialog. */
    candidates: string[];
}

export interface CreateDiagramRequest {
    qname: string;
    type: 'Diagram';
    fields: Record<string, unknown>;
}

export type BuildResult =
    | { ok: true; request: CreateDiagramRequest; tabId: string; displayName: string; kind: string }
    | { ok: false; error: string };

/** Validate the form and build the `POST /api/elements` body. A derived
 * diagram needs a subject that is one of the offered candidates; a blank one
 * is `shapes: {}` (an empty manifest, so Add and Connect build it) and takes
 * an optional subject, used by the connect gesture as the owner of new
 * connections. */
export function buildCreateRequest(form: NewDiagramForm): BuildResult {
    const nameError = validateName(form.name);
    if (nameError) {
        return { ok: false, error: nameError };
    }
    const info = kindInfo(form.kind);
    if (!info) {
        return { ok: false, error: `Choose one of the diagram kinds (${DIAGRAM_KINDS.map(k => k.kind).join(', ')}).` };
    }
    const subject = form.subject.trim();
    if (form.startFrom === 'derive') {
        if (subject === '') {
            return { ok: false, error: `A derived ${info.kind} diagram needs a subject: ${info.hint}.` };
        }
        if (!form.candidates.includes(subject)) {
            return {
                ok: false,
                error: `'${subject}' is not ${info.hint}. Pick one of the suggestions (${info.subjectTypes.join(', ')}).`,
            };
        }
    } else if (subject !== '' && !form.candidates.includes(subject)) {
        return { ok: false, error: `'${subject}' is not ${info.hint}. Leave the subject empty or pick a suggestion.` };
    }
    const fields: Record<string, unknown> = { diagramKind: info.kind };
    if (subject !== '') {
        fields.subject = subject;
    }
    if (form.startFrom === 'blank') {
        fields.shapes = {};
    }
    const name = form.name.trim();
    const qname = joinQname(form.pkg, name);
    return { ok: true, request: { qname, type: 'Diagram', fields }, tabId: tabId(qname), displayName: name, kind: info.kind };
}
