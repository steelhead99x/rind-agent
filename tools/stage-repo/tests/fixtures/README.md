# stage-repo fixtures

`in/` is a captured pair of host JSON files, a log, a note, and a rendered
sheet. `expected/` is the tree `stage-repo.py` wrote from those inputs with:

```
stage-repo --out DIR \
  --json in/alpha.json --rename captured-alpha=alpha \
  --json in/beta.json --rename captured-beta=beta \
  --replace 'Test CPU=Sample CPU' \
  --render in/render \
  --log alpha=in/logs/run.log \
  --note alpha=in/notes/alpha.md
```

Private IPv4 addresses in the inputs are rewritten to `<lan-ip>`. Fixture
files do not contain home-directory paths; the parity tests build those
inputs at runtime.
