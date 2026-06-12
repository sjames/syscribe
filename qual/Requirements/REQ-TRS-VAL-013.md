---
id: REQ-TRS-VAL-013
type: Requirement
name: Tool shall optionally fetch and verify remote sourceFiles via a configured download hook
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** support a download hook, configured under `[remote]` in `<model_root>/.syscribe.toml`, that fetches remote `sourceFile:` URIs into a local cache so they can be verified like local files:

```toml
[remote]
download = "curl -fsSL {url} -o {dest}"
# cache_dir = ".syscribe/cache"   # optional; default shown
```

- The hook **shall** be a POSIX `sh` command with `{url}` and `{dest}` placeholders (substituted as shell-quoted values); the tool **shall** also expose `SYSCRIBE_URL` and `SYSCRIBE_DEST` in the hook's environment.
- Fetched files **shall** be cached (keyed by URL, preserving the URL's file extension) so repeated runs do not re-download.
- For safety, the hook **shall not** run implicitly: defining it has no effect unless the user passes `validate --fetch-remote`. Without that flag, remote sourceFiles remain accepted-but-unverified.
- With `--fetch-remote`: a successfully fetched remote file **shall** be subject to `W009` (function-level checks) against the downloaded copy; a remote file the hook fails to retrieve **shall** raise `W004`.

**Source:** Feature request — download hook for remote files, configured in `.syscribe.toml`. Builds on REQ-TRS-VAL-012.

**Acceptance criteria:** with no `--fetch-remote` flag, a remote `sourceFile` yields neither `W004` nor `W009`; with `--fetch-remote` and a configured hook, a fetched file whose function is missing yields `W009`, and a URL the hook cannot retrieve yields `W004`.
