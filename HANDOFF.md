# Handoff: quickplay-cancel-fix

Updated 2026-11-10 20:40Z (Get-Date). Playbook followed: `ssbu-mod-playbook` `4ca80e1` (the lifecycle change for Ups beside
niced compiles; the rest is the same content as `87b3de0`, read in full for 0.2.0).

- **Branch / commit:** `main`, fast-forwarded to the 0.2.1 branch `v021-rn` (no tag, no Forgejo release: the operator plays
  it first). 0.2.1 = R + N-names (the operator's 19:43Z order):
  - **R:** a Quickplay lobby entered from the character select gives this console's slot the rematch count (1 to 2)
    only while the waiting area is up. On PracticeScene's exit request the native 1 goes back, before the post-battle
    bump, so both consoles count every slot the same. The raise follows the console when its search moves it into
    another host's room.
  - **N-names:** the member pass's count load (`.1563ef8`) reads 2 for an opponent's slot with a count below 2, so the
    opponent's name shows instead of "???". Nothing is written.
- **Why 0.2.1:** v0.2.0 left the own count one battle ahead of the opponent's view (3 vs 2 after match 1). In Eden, the
  first rematch match after a v0.2.0 first match froze the vanilla opponent 2 of 2 times (`qcf-srm12`, `qcf-sfz1`).
  Vanilla ran clean 4 of 4 times (`qcf-srm11`, `qcf-xfz1..3`). The freeze A/B is in the worker's `REPORT.md`
  (2026-11-10 17:35Z onward).
- **Build:** release NRO `ae574e8a09b52b2c…` (36,864 B), built at `nice -n 19` with 2 jobs.
  - Gates: `cargo test` (28), `cargo clippy --all-targets -- -D warnings`, `--no-default-features`, `cargo skyline
    build --release`, and the diag build.
- **Evidence (Eden, research LAN runtime `…-rmcdump-52288a74e874`, b on the GDB stub):**
  - R+N arms `qcf-xfz10` (XPS, TLS on) and `qcf-sfz5` (Surface, TLS off, so the two pairs can't cross-match).
  - Each arm, on a, covers: Leave Battle during the opponent's load from the character select, then a clean leave to
    it; the opponent's name instead of "???"; the yes/yes rematch match played out on both, with no freeze.
  - R arms `qcf-xfz5`, `qcf-xfz7`.
  - Outputs: `/home/intpa/eden-runs/<label>/out` on the XPS and the Surface.
- **Rows:** `eden/make_freeze.py --gdb` (from `eden/obs01` via `make_rematch.py` and the worker's `make_research.py`).
  The arm scripts are `eden/stage-fz.sh`, `fz_chain.sh`, `freeze_watch.sh` and `gdb_sample.py`.
- **Next:**
  - The operator plays 0.2.1 from the card. Then tag and release it if he says so.
  - 0.3.0 (the joiner's ready hold) is on branch `v030-hold`. It needs a rebase onto 0.2.1 and its Eden proof.
- **No secrets** in this repo or its notes.
