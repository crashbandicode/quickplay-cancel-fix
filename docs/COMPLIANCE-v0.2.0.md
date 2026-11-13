# Compliance report: quickplay-cancel-fix

2026-11-10, v0.2.0 (`669f5d9`). 2026-11-13 note: v0.2.0 was withdrawn the same day; its tag and release no longer
exist, and v0.2.1 replaced it. The "Card" row below was true when written (gates at 08:32Z); 0.2.0 was installed
later that day, at 11:45Z on the operator's order, and removed at 17:42Z
(`smash-training-lobby/artifacts/card-install-qcf-0.2.0/`, `card-remove-qcf-0.2.0/`). The release rows describe a
release that is gone.

| Requirement | Tag | Result | Evidence |
| --- | --- | --- | --- |
| crate-type is ["cdylib"] only (no rlib) | M | pass | Cargo.toml crate-type = ['cdylib'] |
| release profile opt-level "s", lto, codegen-units 1, panic abort | M | pass | Cargo.toml [profile.release] as required |
| skyline pinned to =0.6.0 | M | pass | Cargo.toml |
| diag cargo feature declared | M | pass | Cargo.toml [features] |
| toolchain 1.98.1 | M | pass | rust-toolchain.toml |
| runtime only under cfg(target_os = "switch") | M | pass | src/lib.rs |
| supported version 13.0.5 in model | M | pass | src/model.rs |
| every site guard is 64 bytes | M | pass | 5 guard(s) in src/, sizes [64] |
| counting allocator reports heap_allocations | M | pass | src/**/*.rs |
| host tests in model (cargo test) | M | pass | 21 #[test] in src/** |
| README carries the AI-assistance disclaimer | M | pass | README.md |
| README has a GIF, build commands and a license line | M | pass | README.md |
| GIF after.gif: 1280 wide, under 10 s | M | pass | 1280 px, 8.4 s |
| GIF before.gif: 1280 wide, under 10 s | M | pass | 1280 px, 9.5 s |
| dated evidence doc present | M | pass | docs/EVIDENCE.md |
| numbers in the evidence come from tested scripts over the logs | C | pass | tools/: branch_into.py, crop_ref.py, gif_frames.py, gifcheck.py, guard.py, lr_main.py, roi_check.py, test_verdict.py, timeline.py, verdict.py |
| handoff file present | M | pass | HANDOFF.md |
| remote is Forgejo, not GitHub | M | pass | https://git.syntaxbyexample.com/intpa/quickplay-cancel-fix.git |
| working tree clean | M | pass | git status |
| no script mounts, unmounts or ejects the card (the user connects and disconnects it) | M | pass | none in the tracked scripts or the playbook's tools/switch |
| build identity | M | info | HEAD 669f5d9; v0.2.0 |
| shipped NRO identity | M | info | /home/intpa/quickplay-cancel-fix-work/release/assets-v0.2.0/libquickplay_cancel_fix.nro sha256 0b12208d42aa2656 resident 40960 B (40 KiB) |
| no diag strings in the shipped NRO | M | pass | libquickplay_cancel_fix.nro: no diag markers |
| New user-facing refusals cite an approved restriction (// restriction: ID, docs/RESTRICTIONS.md) | M | pass | 0 added refusal(s) since d59b867, all cite approved IDs |
| Design report traces every operator clause to a test that exists | M | pass | 9 clause(s), every test found |
| Gates run: test, clippy -D warnings, no-default-features, skyline release, diag build | M | pass | `gates.ps1` 2026-11-10 08:32Z: cargo test 21 passed, clippy --all-targets -D warnings clean, --no-default-features ok, skyline release `0b12208d…`, diag `bddd7930…`; tools unittest 9 passed |
| Fixes: root cause fixed, or each workaround approved by the operator before it shipped and listed with its date | M | pass | root cause: the per-slot battle count the character select resets and a rematch keeps; the fix sets the own slot to the rematch value. No workaround (docs/EVIDENCE.md "Workarounds: none") |
| Card scan for the site (and vtable slot) over a dated card copy | M | pass | `card-scan-result-020.txt` 2026-11-10 07:23Z, scan-offsets.py over the 20 card plugins (2026-11-08 preflight hashes plus TrainingRooms rc.5/rc.6-f6): nobody at .1851674/.230d000/.2315f40/0x5313fb4; TrainingRooms calls and fingerprints .16f6340 (read-only here); five plugins read the mode word |
| Hook sharing or chaining; both stand-in modes and load orders in Eden | C | N/A | the card scan shows nobody at the hook site .1851674; the hook still chains onto a foreign hook (`chain`, host tests `other_plugins_hooks_are_accepted_and_changes_are_not`, chain::tests) |
| Hook after the prologue or the function called instead | M | pass | .1851674 is in PracticeScene's running state after its update call (`bl .135f570`), past the prologue at .1851660; the most specific instruction found for every waiting-area frame |
| Hooks never block on a lock a plugin thread can hold: lock-reach test passing (fails on a seeded violation), or why none applies | M | N/A | no locks in the plugin (no Mutex/RwLock/lock/Once in src/); the release build starts no thread; the diag build's writer thread drains a lock-free atomic ring the hook only pushes to |
| Shipped NRO writes only its install line (plus a logging plugin's data files) | M | pass | qcf-xrm08 and qcf-xrm09 a-quickplay-cancel-fix-events.log: one line each, `plugin=quickplay-cancel-fix/0.2.0 wait=stock/near result=installed heap_allocations=0` |
| Online: read-only rules reviewed hook by hook | C | pass | docs/EVIDENCE.md "Online review": no random number drawn, no game code skipped or reordered, one lobby count written (not fighter/stage/round state) |
| Online: replay A/B with the desync detector identical | C | N/A | the hook is in the waiting area (PracticeScene), which a replay never enters; the change is a lobby count before the match (docs/EVIDENCE.md "Online review") |
| Online: two consoles past the changed path, desync_seen=0 at the same frame | C | pass | qcf-xrm07/08/09: matches after the change with the plugin hosting (5) and joining (6) played to the end, the same sessions and end frames on both consoles, desync_seen=0 (`tools/verdict.py`); the two `cut` sessions are the harness idle during a GIF encode (docs/EVIDENCE.md) |
| Eden: claim written and released, also after a failed run | M | pass | ledger CLAIMED/RELEASED per run (tr-opus-qcf host=xps/surface) and the integration inbox: qcf-xrm06 06:10-06:29Z, xrm07 06:41-07:09Z, xrm08 07:14-07:58Z, xrm09 08:00-08:28Z |
| Eden: at most two instances per Linux host, one on the Ally; MemAvailable >= 10 GiB; not profile a or :61 | M | pass | XPS pair (two instances, displays :124/:125), preflight `eden slots` limit 2 (refused staging xrm09 while xrm08 ran); MemAvailable 27-28 GB at Up, >= 13 GB during runs; profile copies, not profile a |
| Eden: profile copy with hard-linked game content; only the copy changed | M | pass | eden_test.py up builds work/profiles/profile-{a,b} as copies (game content linked); the Prune line in each RESULTS.md: "4.2 GB freed of 41.0 GB of copies" |
| Copies: byte-identical files linked, differing or written-in-place files copied | M | pass | the runner's prep (eden_test.py up): 41 GB apparent per run, 4.2 GB real copies (RESULTS.md Prune) |
| Disk: one target dir per source copy (never shared between copies), cargo home shared per host and toolchain, df checked (>= 40 GB), finished build products deleted with a manifest | M | pass | build/targets/{host,release,diag} for the repo, release/target-v0.2.0 for the tag clone; one cargo home (toolchain/tools/cargo); df 397 GB free (Surface) before the builds, 116-119 GB (XPS) around the last Ups; the tag clone and its target (274 MB) deleted after the release, listed in release/deleted-v0.2.0.txt (NRO kept in the assets) |
| Eden: waits gated on a log line or screenshot check | M | pass | steps.py waits on trace states (parent/child, PracticeScene result), the detectors' sync_end and screen checks (tab, dialog focus, CSS); fixed sleeps only between button presses |
| Eden: test run plays a runner dry-run before Up (recorded segments + sha256, automated gates, per-row receipts, rows chained on end state); manual input only to build a missing segment | M | pass | dry-run READY before each stage (rows-af01, rows-af02, rows-rm06, rows-lt01); per-row receipts in out/*.json; manual_inputs 0 in every pace line |
| Eden: rows grouped into as few boots as hosts/builds allow; a failed row is fixed or deferred on the same boot; per-run RESULTS.md <= 20 lines, compliance once per phase | M | pass | four boots for four questions (reference, diag, release repeat, later match + takes); 0 failed rows; RESULTS.md 9 lines each; this report once for the 0.2.0 phase |
| Eden: failed precondition aborted and retried under the next label | M | pass | qcf-srm07 refused at preflight (a cargo check on the Surface), retried as qcf-srm08 |
| Eden: game-side facts from guest tracing (persistent trace.* points or the one-shot tracepoint) on a run already happening; diag rebuild or extra trace boot only with a stated reason | M | pass | the per-slot counts, HUD table and mode word came from samplers on runs that were happening anyway (srm08, xrm06); the diag build (xrm07) only to log the adopted count |
| Eden: profile restored, temp files removed, host released | M | pass | down.sh: work/, state/, inputs/ removed after the logs were copied to out/logs; released=True in each RESULTS.md |
| Eden: the run's copies pruned at Down once its files were in out/ (the Prune line); Up refused below 60 GB free | M | pass | RESULTS.md Prune lines (xrm06-09: 4.2 GB freed, run-written files in out/post-run-*/); XPS 116-119 GB free around xrm08/xrm09 (the runner refuses Up below 60 GB) |
| Eden: console.log searched for panic/abort/crash; run.json and memory.peak kept | M | pass | xrm06-09 a/b console.log and trace streams: 0 panic, abort, crash, unmapped; out/logs/*-run.json and *-memory.peak kept |
| Every local command under the memory and time guard | M | pass | builds and Ghidra under systemd-run MemoryMax=6G plus timeout; every shell call with a timeout; scripts with ${var} in files (gates.ps1, rebuild.ps1, xps/*.sh) |
| Evidence doc states run labels, build, what was done, what the log showed | M | pass | docs/EVIDENCE.md: qcf-xrm06..09, NRO sha256s, the rows, tools/verdict.py output |
| GIF caption gives date and build; clip shows start, action and result | M | pass | README captions: Eden 2026-11-10, qcf-xrm09, build 0b12208d…; before: CSS, join, menu without Leave Battle; after: CSS, join, Leave Battle, Yes, CSS; frames reviewed (docs/EVIDENCE.md "README GIFs") |
| No operator quotes, no claims beyond the tests | M | pass | README, EVIDENCE and the release notes quote nobody; "not yet tested on a Switch" stated |
| Handoff file current (branch, commit, evidence, live state, next commands) | M | pass | HANDOFF.md (main, 0.2.0, evidence, rows and analysis commands, next: the card on order) |
| Release: version bump, annotated tag, assets verified by anonymous download | C | pass | v0.2.0 (annotated, 669f5d9), release id 662: NRO, manifest.json, SHA256SUMS.txt, before.gif, after.gif, each ok by anonymous download |
| Release: rebuild of the tag byte-identical to the released NRO | C | pass | fresh clone of v0.2.0, own target dir: 0b12208d42aa2656290d95cca6906fb0ad173c2611e9793fda6058c5040a876d = the tested NRO |
| Card: MTP preflight, rollback copy, readbacks, receipts not re-run | C | N/A | not installed on the card: it needs the operator's own order (brief); ready for the card |
| Card: the user connected and disconnected it; the agent never mounted, unmounted, ejected or remounted it, nor started or stopped DBI | M | pass | the card was not touched |
| Nothing pushed, tagged or written to the card without the operator's order | M | pass | push, tag and release per the brief's shipping order (operator, once proven); nothing on the card |
| Model/effort gate and watcher per the routing authority | M | N/A | a worker session assigned by the primary; the gate and watcher are the primary's |

