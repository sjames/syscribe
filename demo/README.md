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

## The recording

`guarded-write.cast` (asciicast v2) and `guarded-write.gif` are a recording of this script driving the `syscribe`
built from the same commit as the README's transcript. Regenerate both whenever the script or the delta output changes:

```bash
pip install asciinema                                  # 2.x; the GIF needs https://github.com/asciinema/agg
asciinema rec --overwrite --cols 122 --rows 26 -i 2 \
  -c "env DEMO_PACE=0.7 python3 demo/mcp-guarded-write.py" demo/guarded-write.cast
agg --font-size 15 --last-frame-duration 5 --theme monokai demo/guarded-write.cast demo/guarded-write.gif
```

The GIF is embedded in `README.md`. To host the cast on asciinema.org instead, `asciinema upload demo/guarded-write.cast`
and link the result (this publishes it, so it is not done automatically).
