# Handoff: quickplay-cancel-fix

Updated 2026-11-10 23:55Z (Get-Date). Playbook followed: `ssbu-mod-playbook` `4ca80e1`.

- **Branch / commit:** `main`, fast-forwarded to `v030-hold`.
  - Tag `v0.3.0`: a Forgejo **pre-release** until the operator has played the hold on his Switch.
  - Tag `v0.2.1` (`fe0aa0e`): a normal release; the operator played it on the Switch.
- **0.3.0 = 0.2.1 + the joiner's ready hold:**
  - **The hold:** on every Quickplay match this console joins (from the character select or a rematch), the waiting
    area's ready report (`.151b044`, `bl .168db60` with 7) reports 6. After `hold_seconds` of frames in which the
    waiting menu could open, the plugin calls `.168db60(svc, 7)`.
    - A started round or Leave Battle (`.1851930`) ends it.
    - So do 600 frames in which the menu couldn't open.
  - **`config.ini`:** `hold_seconds`, 0.5–10, `0` = off, default 3. It's read at each joined match.
  - **The own slot** now comes from the lobby's member table (`0x5308760`, stride 0x1c8, station at `+0x10`) first,
    else the slot table. For a held joiner, the slot table only gets its station after the ready report.
- **Build:** release NRO `d69f11cf17c618a1…` (53,248 B), built at `nice -n 19` with 2 jobs.
  - Gates: `cargo test` (41), `cargo clippy --all-targets -- -D warnings`, `--no-default-features`, `cargo skyline
    build --release`, and the diag build.
- **Evidence:** [docs/EVIDENCE.md](docs/EVIDENCE.md).
  - The main run is `qcf-xhd4` (XPS, 17/17 rows): holds of 3, 6 and 10 s, leaves during holds, held matches and a
    rematch after one played out, and the hold off as the before.
  - Outputs: `/home/intpa/eden-runs/qcf-xhd4/out` on the XPS.
- **Rows:** `eden/make_hold.py` builds `rows-hd04` from the freeze rows; `eden/stage-hd.sh` stages them on the XPS. The
  hold steps are in `eden/obs01/steps.py`.
- **Next:**
  - The operator plays 0.3.0 from the card; then the pre-release flag comes off.
  - Open ideas:
    - keep the hold while the waiting menu is open (no "menu open" signal found yet);
    - a smaller member-test guard, so the install line loses `other_hook=member_test`;
    - remember a Pause press during the loading screen.
- **No secrets** in this repo or its notes.
