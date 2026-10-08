# MCP memory use on large models

Status: roadmap note, 2026-10-08. Not yet decided or built.

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
