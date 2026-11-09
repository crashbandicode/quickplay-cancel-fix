# Handoff: quickplay-cancel-fix

Updated 2026-11-09 16:40Z (Get-Date).

- **Branch / commit:** `main`, last commit `d59b867` (skeleton). Nothing pushed; no Forgejo repo yet.
- **State:** design pending the observe boot `qcf-obs01` (XPS Quickplay pair, staged and READY; slot after rc.6's
  XPS queue). The skeleton implements design A (the host's full-room wait at `.15669c4`); the joiner-side design B
  is decided from obs01's trace.
- **Build:** release and diag NROs from the Surface gates burst 16:17Z (release `13e07c78…`, diag `9bfc0599…`,
  36,864 B each); `cargo test` 21 passed, clippy clean.
- **Evidence so far:** static RE notes in the worker's report; card scan clean for `15669c4 1566650 16b7570` over all
  18 card plugins (2026-11-08 preflight hashes).
- **Next commands:**
  - XPS, after integration's "XPS free for QCF" note:
    `python3 -B -I /home/intpa/pb-qcf-45d0fcb/tools/eden-test/eden_test.py up /home/intpa/eden-runs/qcf-obs01`,
    then `run`, then `down`; RELEASED in the integration inbox; remove the run's `work/` and the inputs.
  - Analysis: `python3 tools/timeline.py <trace.jsonl>` and
    `python3 tools/verdict.py --trace a=… --trace b=… --detector a=… --detector b=…`.
- **No secrets** in this repo or its notes.
