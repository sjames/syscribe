# Recording the guarded-write demo

`mcp-guarded-write.py` replays the README demo against a throwaway copy of `model_auto/`: it starts a real
`syscribe mcp` server over stdio and plays the agent (propose a requirement with a dangling reference → dry-run
shows the error → commit is refused → fix → commit succeeds). Python 3 standard library only. Your working tree
is never touched.

```bash
cargo build -p syscribe                     # or: python3 demo/mcp-guarded-write.py /path/to/syscribe
python3 demo/mcp-guarded-write.py           # instant output
DEMO_PACE=0.8 python3 demo/mcp-guarded-write.py   # paced for a recording
```

## Recording it

Nothing here is recorded yet. With [asciinema](https://asciinema.org):

```bash
asciinema rec --cols 110 --rows 30 -c "env DEMO_PACE=0.8 python3 demo/mcp-guarded-write.py" demo.cast
asciinema upload demo.cast      # prints https://asciinema.org/a/<ID>
```

To make a GIF for places that don't embed asciicasts (Show HN, social), convert with
[agg](https://github.com/asciinema/agg): `agg demo.cast demo.gif`.

Then, in `README.md`, replace the `TODO(demo)` comment under **See it work** with

```markdown
[![asciicast](https://asciinema.org/a/<ID>.svg)](https://asciinema.org/a/<ID>)
```

and paste the same link (or the GIF) into `SHOW_HN.md`.
