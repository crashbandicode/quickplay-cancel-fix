# Handoff: quickplay-cancel-fix

Updated 2026-11-10 08:10Z (Get-Date).

- **Branch / commit:** `main`. 0.2.0 (`e5b154b`) is the shipped design: a Quickplay lobby entered from the character
  select takes the rematch lobby's battle count for this console's slot (1 to 2). Designs A and B are removed.
- **Build:** release NRO `0b12208d42aa2656…`, diag `bddd7930ffa2f546…`, 36,864 B each.
  - Gates: `cargo test` (21), `cargo clippy --all-targets -- -D warnings`, `--no-default-features`, `cargo skyline
    build --release`, and the diag build.
  - `tools/`: `python3 -m unittest test_verdict` (8).
- **Evidence:** [docs/EVIDENCE.md](docs/EVIDENCE.md).
  - Eden runs `qcf-xrm06` (the game's own leave, reference), `qcf-xrm07` (diag build), `qcf-xrm08` (release build) and
    `qcf-xrm09` (a later match's leave with the plugin installed, plus the README takes).
  - The logs are under `/home/intpa/eden-runs/<label>/out/logs` on the Surface. The design report with the requirement
    trace is the worker's `DESIGN.md` (working file, not shipped).
- **Analysis:** `python3 tools/verdict.py --tries <run>/out --plugin a=… --detector a=… --detector b=…`.
- **Rows:** generated from `eden/obs01` by `eden/make_rematch.py`, then the worker's `lan2c/make_after.py` /
  `make_later.py` (copies in `eden/`). The config comes from `eden/make_config.py` (`--nro` repeatable).
- **Next:** the Forgejo release `v0.2.0` (NRO, manifest.json, SHA256SUMS.txt, GIFs). Install on the operator's card
  only on the operator's order.
- **No secrets** in this repo or its notes.
