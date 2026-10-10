# import-reqif — create Requirements from a ReqIF document

## SYNOPSIS
    syscribe -m <root> import-reqif <file.reqif> [--into <package>] [--id-prefix <PFX>]
        [--class <reqClass>] [--domain <reqDomain>] [--update] [--dry-run]

## DESCRIPTION
Reads the SPEC-OBJECTs of a ReqIF document (an OEM export, or a file written by
`export-reqif`) and creates one native `Requirement` per object. Objects typed
`Package` or `TestCase` (folders, tests) are skipped.

Each requirement is `status: draft`, takes its name from the `NAME` / `ReqIF.Name` /
`ReqIF.ChapterName` / `Title` attribute (else the object's `LONG-NAME`) and its body
from the `DESC` / `ReqIF.Text` / `Text` / `Description` attribute reduced to plain
paragraphs (an object without text gets the placeholder "No text in the ReqIF source." because an empty
body is `E012`). The OEM identifier (`ReqIF.ForeignID`, `ID` or `SYSCRIBE_ID`, else the
object IDENTIFIER) is kept in `extRef: ["reqif:<id>"]`. Ids are `<PFX>-NNN`, numbered
after any existing id with that prefix.

An object whose identifier repeats within the file is skipped (reported as `duplicate`). A
SYSCRIBE_ID that is already the id of a requirement matches it, so an `export-reqif` file imports
back onto its own model without creating anything.

Re-importing is idempotent: an object whose `reqif:<id>` already exists is reported
as `exists` and left alone. With `--update` its name and body are rewritten when they
differ — every other frontmatter field is kept. A body that has headings or code fences of its
own (it was worked on after import) is kept and only the name is refreshed. Only requirements
stored in their own `<id>.md` file with LF line endings are rewritten; others are reported as
failed and the exit code is 1.
A changed text is then picked up by the normal suspect-link / baseline machinery.

Not yet: CSV/Excel mapping files, ReqIF relations and attribute enumerations
(status/domain), and nested SPEC-HIERARCHY folders (GH #241).

## OPTIONS
    --into <package>   Target package (default Requirements).
    --id-prefix <PFX>  Stable-id prefix (default REQ-IMP).
    --class <c>        reqClass of created requirements (default stakeholder).
    --domain <d>       reqDomain of created requirements (default system).
    --update           Rewrite name and body of already-imported requirements.
    --dry-run          List what would be created or updated; write nothing.

Exit 0 · 1 on an unreadable or malformed file, a document without requirement
objects, or a write failure.

## SEE ALSO
    export-reqif, set, validate
