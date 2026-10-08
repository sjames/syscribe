# MCP memory use on large models

Status: rounds one (0.46.0) and two (0.48.0) built, 2026-10-08. Bodies on disk and the watcher's overlap remain, with the reasons given at the end.

## Report

The MCP server was restarted while serving a model of about 12,000 elements. The cause is not
confirmed: it may be the operating system or a container killing the process for memory, or a
crash in a reload. The first roadmap step is therefore to **measure and log**, not to optimise.

## What is known

A one-off `validate` of a synthetic 12,000-element model (4,000 each of requirements, part
definitions and test cases, about 1 KB of body each) peaks at about 133 MB resident in a release
build. A single copy is not the problem. The MCP server holds more than one copy at times:

| Moment | Copies of the model in memory |
|---|---|
| Steady state | the element list, the graph, the node index and the resolver indices |
| A reload after the watcher fires | the old store and the new store, both alive until the swap |
| A guarded write (create, update, delete, move) | the live store, plus a candidate copy of the whole model walked and validated |
| Two writes or a write during a reload | the above, stacked |

A model several times larger than the synthetic one, with long bodies and many inline features,
makes each copy several times larger, so a stacked peak of three to four copies is plausible
territory for a memory-limited host.

## Ideas, in the order worth trying

1. **Measure first.** Log resident memory at startup, after each load, reload and guarded write,
   and expose it in a `server_stats` MCP tool and on exit. Add a 12,000-element benchmark to CI
   with a memory ceiling. Without this every later idea is a guess.
2. **Never hold two stores.** Build the replacement store, but drop the old one's large parts
   first, or serialise reloads so a second reload never starts while one is running. Saves up to
   one full copy at reload.
3. **Do not copy the model for a guarded write.** The candidate copy re-walks every file on disk.
   Instead validate an overlay: the live element list with only the changed elements replaced,
   and re-run only the checks that the change can affect. Saves a full copy per write and most of
   its time.
4. **Keep bodies on disk.** `doc` (the Markdown body) is usually the largest field and is needed
   only by `show`, search and the web panel. Store the file offset and length, read on demand, and
   keep a small bounded cache. Frontmatter and links stay in memory.
5. **Intern repeated strings.** Qualified names, ids, types and status values are cloned into the
   element list, the node index and every resolver map. Interning (one shared allocation, cheap
   handles) removes most of those duplicates.
6. **Trim per-element overhead.** Most elements use a handful of the many optional frontmatter
   fields, and unknown keys sit in a generic YAML value tree. Measure the size of one element,
   then move rarely-set fields behind one boxed struct or a compact representation.
7. **Load lazily by subtree.** For very large models, load the index (names, ids, links) at start
   and parse a package's bodies and inline features on first use. Larger change; do it only if
   items 1 to 6 do not suffice.
8. **Bound the process.** An optional memory limit that refuses a write or defers a reload with a
   clear error, instead of the host killing the server.

## Decision to make

Items 2, 3 and 4 give the most for the least risk and keep behaviour identical. Items 5 to 7 are
deeper changes to core types and should wait for the measurements from item 1.

## What was built (0.46.0) and measured

Measured on a synthetic 12,000-element model (4,000 requirements, part definitions and test cases, about 1 KB of body each), release build, driving `syscribe mcp` over stdio:

| | Before | After |
|---|---|---|
| Resident after load | 121 MB | 62 MB |
| After three guarded writes and a reload | 261 MB resident, 342 MB peak | 68 MB resident, 126 MB peak |
| Growth per guarded write | 27 to 40 MB, never released | none |
| One in-memory element record | 7,008 bytes | 1,792 bytes |

What changed:

- **Compact element records.** An element's frontmatter held 271 optional fields inline, about 6.8 KB of mostly empty slots. The 55 commonly used fields stay inline. The other 216 sit in three tiers boxed behind `Deref`, allocated only when an element sets one of them, so an element that uses none pays one pointer and every field is still read and written as before. A file is read through one flat wire struct and split into tiers, because nested `flatten` would leave those keys visible to the unknown-field catch-all.
- **No growth across writes.** The growth was glibc keeping a separate arena for each worker thread and never returning freed pages. On Linux with glibc the server now caps the arena count at startup and trims free memory after every rebuild. A different allocator (mimalloc) was tried and measured worse, 107 MB after load and 221 MB after one write, so it was not adopted.
- **Observable.** The `server_stats` tool reports element count, body bytes, bytes per element record, and resident and peak resident memory. The watcher's reload log line carries the resident size.
- **Guarded by a test.** `crates/syscribe/tests/mcp_memory.rs` loads 12,000 elements and fails if resident memory exceeds 160 MB, if three guarded writes push the peak past 300 MB, if memory keeps growing across writes, or if an element record grows back toward its old size.

The budget the project commits to: a 12,000-element model of this shape needs about 85 MB at peak and 65 MB at rest. Models with long bodies or many inline features need more, in proportion to their size.

## Round two (0.48.0)

| | After round one | After round two |
|---|---|---|
| Peak across three guarded writes and a reload | 126 MB | 85 MB |
| One guarded write | 2.4 s | 1.7 s |

- **One model at a time during a write.** A guarded write holds the store's write lock, so nothing reads the live model while the candidate is built. For a model of 3,000 elements or more the live copy is therefore dropped first, the candidate is built and validated, and the live model is reloaded from disk after the write (a commit) or rebuilt (a dry run or a refusal). The peak is one model plus the validator's working memory, not two models.
- **The live model's findings are kept.** A write compares the candidate's findings with the live model's. The live model's are no longer recomputed by validating it in memory before every write: they are kept in the store, and a commit stores the candidate's findings as the new model's, so only the first write after a load or an external reload pays for them. This is also what let the live model be dropped.
- **Both stores.** The MCP server (`McpStore`) and the web server (`ModelStore`) share the model-layer functions `compute_baseline` and `guarded_write_cached`.

## Still open

- **The file watcher still builds the replacement model beside the live one.** It must: the live model keeps answering readers while the new one loads, and a half-saved file is detected by comparing the two. A model of 12,000 elements peaks near twice its size for the fraction of a second a reload takes. Avoiding it would need a cheap pre-scan for torn files before the live model is dropped, at the cost of serving nothing during the reload.
- **Bodies stay in memory.** A body is read by validation, search and the web panel, so loading one lazily only helps until the first write. Keeping bodies on disk behind a bounded cache needs every reader of `RawElement::doc` (hundreds of sites) to take a handle instead of a string, in exchange for the body share of memory (about a fifth at 1 KB per element, more for long documents).
- **Platforms.** The arena cap, trimming and memory figures apply to Linux with glibc. Other platforms get the smaller records and the one-model write path.
