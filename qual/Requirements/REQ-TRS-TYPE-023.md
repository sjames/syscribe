---
id: REQ-TRS-TYPE-023
type: Requirement
name: Tool shall flag a peer-repo ref that disagrees with its git submodule gitlink
status: draft
reqDomain: software
verificationMethod: test
---

When a peer repository's `path` (§14, GH #62) is a **git submodule** of the composing model's
repository, the parent repo records a pinned commit (the **gitlink**) for that path. The tool
**shall** compare the commit a repo's `ref:` resolves to against that gitlink and **shall**
emit warning **`W512`** when they disagree, catching a `.syscribe.toml` `ref:` that has
silently diverged from the `.gitmodules` pin.

- For each `[repos]` entry whose `path` resolves to a **submodule gitlink** in the parent
  repository's tree (`HEAD`), the tool **shall** resolve the entry's `ref:` to a commit and
  compare it with the gitlink commit; a mismatch **shall** raise `W512` naming the repo, the
  resolved `ref:` commit, and the gitlink commit.
- `W512` **shall** be gateable to a hard failure with `--deny W512`, consistent with `W510`/
  `W511`. It is **not** raised when the comparison cannot be made — `path` is not a submodule
  (a sibling checkout or monorepo path), no `ref:` is configured, the parent is not a git
  repository, git is unavailable, or either commit does not resolve — so it never false-flags
  a non-submodule composition.
- `W512` is **independent** of `W511`: `W511` compares the peer work tree's `HEAD` to the
  `ref:`; `W512` compares the parent's gitlink pin to the `ref:`.

### Validation rules

| Code | Condition |
|---|---|
| `W512` | A peer repo's `ref:` resolves to a different commit than the git submodule gitlink the parent repo records for its `path` (opt-in, `--deny W512`). |

**Source:** §14 (Multi-Repository Model Composition), GH #62.

**Acceptance criteria:**

- A submodule peer whose `ref:` resolves to a commit other than the recorded gitlink emits
  `W512`; `--deny W512` exits non-zero.
- A submodule peer whose `ref:` resolves to the gitlink commit is silent.
- A non-submodule peer (sibling checkout) never emits `W512`.
