# Evidence: quickplay-cancel-fix

Newest version first. Eden runs are driven by the playbook's Eden runner from a `rows.json`. The receipts, frames
and logs stay with each run under its label (`/home/intpa/eden-runs/<label>/out` on the host named).

## 0.3.0: the joiner's ready hold (2026-11-10)

- **Build:** release NRO `d69f11cf17c618a11b86ee7373c5b786af0d53222dce0a8cb998da6bbe1da9e8` (53,248 B), rebuilt
  byte-identical from the tag.
  - Gates: `cargo test` 41, `cargo clippy --all-targets -- -D warnings`, `--no-default-features`, the release and diag
    builds.
- **Setup:** Eden on the XPS, two instances of 13.0.5 playing Quickplay against each other over LAN Online (TLS on, a
  matchmaking group of their own).
  - a runs the plugin; b runs no QCF build and is the host.
  - Both run ssbu-desync-detector, ssbu-tile-cursor, and ssbu-cpu-pilot 0.2.2 (`auto_rematch=on`: it answers
    "Rematch?" with Yes, nothing else).
  - Both are traced (the member-state setter `.168db60`, PracticeScene's menu gate, the name rule `.1563f04`, b's
    countdown and round start).
  - The rows (`eden/make_hold.py`, steps `hold_join`, `hold_rematch`, `expect_hold`, `hold_verdict` in
    `eden/obs01/steps.py`) write a's `config.ini` before each match.

### `qcf-xhd4` (Up 22:58:08Z, DONE 23:29:04Z, 17 of 17 rows, the `v-hold` verdict included)

| row | hold | a's ready report → the plugin's 7 | Leave Battle in the menu | name rule while held | outcome |
| --- | --- | --- | --- | --- | --- |
| `h3l` | 3 s | held as 6; 7 after 3.32 s (Yes at +3.29 s) | +1.18 s | loaded 2 | left to the character select |
| `h3` | 3 s, b searching anew | 2.92 s | +1.42 s | 2 | match: time-out (frame 7542) and sudden death (1937) on both, `desync_seen=0` |
| `ry3` | 3 s, the rematch after `h3` | 2.93 s | +1.55 s | 2 | rematch: 7542 and sudden death 1899 on both, `desync_seen=0` |
| `ry3l` | 3 s, the next rematch | left during the hold | +1.18 s | 3 | left |
| `h6p` | 6 s, b 72.6 s in its room | 6.00 s | +1.57 s | 2 | match: 7542 and 1899, `desync_seen=0` |
| `ry6l` | 6 s, rematch | left during the hold (no 7 sent) | +1.18 s | 2 | left |
| `h10` | 10 s, b searching anew | 10.06 s | +1.44 s | 2 | match: 7542 and 1870, `desync_seen=0` |
| `ry10l` | 10 s, rematch | left during the hold | +1.18 s | 2 | left |
| `h0` | off (`hold_seconds=0`) | not held (7 at once) | +1.18 s | 2 | "Get Ready" closed the leave dialog; match: 7542 and 1876 |

- **The opponent (b, no QCF):**
  - It showed a as "Preparing…" during every hold. In `h10` its countdown ran 12 → 8 meanwhile.
  - After a leave: "Later!" for a's slot, then it searched alone (`ensure_searching` read its PracticeScene state).
  - It never kicked or timed out a held member.
- **The end rows:** `trace.unmapped_access` 0 on both. Both `console.log`s have no panic, abort, crash or unmapped line.
  The install line: `plugin=quickplay-cancel-fix/0.3.0 wait=stock/near names=stock/near leave=stock/near
  ready=stock/near hold_frames=180 other_hook=member_test result=installed heap_allocations=0`.
  - `other_hook=member_test` is the plugin's own names hook inside the member-test guard. The line is written after
    the hooks are in.

### Earlier runs of this version

- **`qcf-shd1`** (Surface, 0.3.0's first build `d2085676`): the hold worked (3.03 s), but the menu during it had no
  Leave Battle. The plugin took its own slot from the slot table (`0x5313f08 + slot * 0xb0`), which gets a joiner's
  station only after its ready report. The fix takes it from the lobby's member table (`0x5308760`, stride 0x1c8,
  station at `+0x10`), as the member pass's own test `.179ed00` does.
- **`qcf-xhd2`, `qcf-xhd3`** (this NRO): Leave Battle and the name were there during the hold. The harness then had
  two of its own misses, both fixed for `xhd4`:
  - Yes came 0.24 s after a 3 s hold had ended. The fix: Plus as soon as the menu gate opens, then the fast leave.
  - A battle step read the previous sudden death's `sync_end` and returned at once. The idle rematch was then
    disconnected for inactivity. The fix: `battle_out` waits only for a later `sync_end`.

### Why a joiner can't leave without the hold

- In `qcf-xfz10` m1 (0.2.1, a joining), PracticeScene's menu gate (`.18512c0`, verdict at `.18512f8`) was open from
  0.15 s after a's ready report until the round, 9.4 s later.
- b was already at 7, so a's own 7 completed the room. Its next frame (+1.8 s) shows "Get Ready", and six Pause
  presses opened nothing.
- With the hold, a stays at 6, the room doesn't reach "every member 7", and the menu stays usable.

### README GIFs (0.3.0)

- Both clips are the joining console's screen (a), 1280×720: the runner's raw takes, trimmed with
  `tools/gif_trim.py` (frame durations kept).
- `after.gif`: `qcf-xhd4` row `h3l` (0.3.0, the 3 s hold), 0.3–7.0 s of the take, 6.7 s.
  - Frames: the waiting area fading in; Pause with Leave Battle; Leave Battle focused (a "Preparing…", the opponent
    "Joined!"); "Would you like to leave the game?" on No, then Yes; then the load back.
- `before.gif`: `qcf-shv1` row `vb` (2026-11-11, **no plugin on either console**), 0.4–9.8 s, 9.4 s.
  - The 0.3.0 release first carried a take of 0.3.0 with the hold off (`qcf-xhd4` row `h0`). That isn't the game
    without the plugin: 0.3.0 still offers a brief Leave Battle there. It was replaced (the operator's correction,
    2026-11-11).
  - Frames: the waiting area with both players "Ready!" (the opponent "???"), "Get Ready" at once, "Closed" slots,
    "Fight Time", the load, and the match's VS screen.
  - a pressed Pause four times (1.2, 2.6, 3.9 and 5.3 s after its ready report), and no menu opened.

### `qcf-shv1`: the joiner without the plugin (Surface, 2026-11-11, 8 of 8 rows)

- Both instances had no QCF build (detector and tile cursor only). LAN TLS was off, a group of its own.
- b searched alone. a joined b's room from the character select and reported 6, then 7, at its waiting area's start
  (natively, from `.151b048`).
- The room went to "Get Ready" at once. a's Pause presses opened nothing (no menu in any look). The match started
  6.6 s after a's waiting area and played to its end on both consoles (frame 7521, `desync_seen=0`).
- Down 00:26:11Z, released.

## 0.2.1 (2026-11-10)

- **Build:** release NRO `ae574e8a09b52b2cbb4a6fbde9643968cfd5434ceff2590e3dfc4720dfce5db3` (36,864 B), tag
  `v0.2.1`. Installed on the operator's Switch and played: it worked as expected.
- **The freeze A/B** (why 0.2.1 replaces v0.2.0):
  - v0.2.0 on a froze the vanilla opponent at the first rematch match after a first match 2 of 2 times (`qcf-srm12`,
    `qcf-sfz1`).
  - With no plugin, 4 of 4 runs were clean (`qcf-srm11`, `qcf-xfz1..3`).
  - v0.2.0 left a's own count at 2, so after match 1 a counted its own slot 3 while b counted it 2.
- **0.2.1** puts the native 1 back on PracticeScene's exit request. In `qcf-xfz10` (XPS) and `qcf-sfz5` (Surface, a
  LAN group of its own):
  - Hosting from the character select: Leave Battle while b loaded, Yes back to the character select. b's name showed
    instead of "???".
  - The Rematch Yes/Yes match played to its end on both (frame 7542, `desync_seen=0`), with no freeze.
  - Each console's opponent was its own pair partner (the runtime's `[lanstations]` PIDs).

## 0.2.0 (2026-11-10, withdrawn)

2026-11-10. Eden on one host (the XPS), two instances of Super Smash Bros.
Ultimate 13.0.5 playing Quickplay against each other over LAN Online (TLS
on, a matchmaking group of their own). Console **a** runs the plugin, console
**b** runs no QCF build. Both run ssbu-desync-detector and ssbu-tile-cursor.
Each run is driven by the playbook's Eden runner from a `rows.json`; the
receipts, frames and logs stay with the run under its label.

- **Builds:**
  - Release NRO `0b12208d42aa2656290d95cca6906fb0ad173c2611e9793fda6058c5040a876d`.
  - Diagnostic NRO `bddd7930ffa2f546e4846d8c62916edc0ab914cb192cad8337ece7383348fc14` (`--features diag`).
  - 36,864 bytes each, built from `e5b154b`; `src/` is the same at the release tag.
- **Extra plugin on a in `qcf-xrm06` and `qcf-xrm09`:** ssbu-cpu-pilot 0.2.2.
  - NRO sha256 `ed8f308f695842a9e5f1728f829cfe8e325c0aa98dee79c875358ad435d7dfea`, as its v0.2.2 manifest gives it.
  - `config.ini`: `p1=player`, `level=1`, `auto_rematch=on`, `auto_save_replay=off`. It installed with `auto_rematch=1`
    and piloted nothing.
  - It only answers the "Rematch?" prompt with Yes, as the game's own answer.
  - Its only rematch setting is Off/On (Off presses nothing), so b's No was the harness's press.
  - The same `config.ini` was copied to b's profile, where no pilot NRO loads it.
- **Numbers** below come from `tools/verdict.py` over each run's logs (host tests in `tools/test_verdict.py`):

  ```text
  python3 tools/verdict.py --tries out --plugin a=out/logs/a-quickplay-cancel-fix-events.log \
      --detector a=out/logs/a-ssbu-desync-detector-events.log --detector b=out/logs/b-ssbu-desync-detector-events.log
  ```

  - **Tries:** the harness pauses during the opponent's load and reads the menu. A try is `shown` when a frame with the
    menu open has the Leave Battle tab, and `left` when the leave then reached the character select.
  - **Sessions:** the detector's online sessions per console:
    - `clean`: a `sync_end` with `desync_seen=0`;
    - `cut`: a frame gap and no `sync_end`;
    - `desync`: a `sync_end` with `desync_seen` above 0.

### What decides the leave

- Quickplay keeps a count of the battles this console has played per member slot of the room (`0x5313fb4 + slot *
  0xb0`, with the slot's station at `0x5313f08 + slot * 0xb0`).
- Entering the lobby from the character select resets every slot to 1 (`.230d000`). Each battle adds 1 to every
  present slot (`.2315f40`). A member that leaves has its slot reset to 1.
- "Rematch?" Yes returns to the lobby without the reset.
- Sampled in `qcf-xrm06`, without the plugin:

  | lobby | this console's slot | the joiner's slot | Leave Battle while the joiner loads |
  | --- | --- | --- | --- |
  | m0, after Rematch Yes | 2 | 1 | shown |
  | m5, after Rematch Yes | 2 | 1 | shown |
  | m3, from the character select | 1 | 1 | hidden |

- The plugin writes 2 into this console's own slot of a Quickplay lobby when it holds 1, and nothing else.
- **Workarounds:** none. There is no guard, skip, restriction or tolerance beyond the 13.0.5 version and byte checks
  at install.

### Runs

| Run | a | What | Pause during the opponent's load | Matches (detector sessions per console) |
| --- | --- | --- | --- | --- |
| `qcf-xrm06` | no QCF (the CPU pilot answers Rematch Yes) | the game's own leave, as reference | m0, m5 (rematch lobbies): shown 1.27 s, `left`; m3 (character-select lobby): hidden | 4 sessions, 4 clean |
| `qcf-xrm07` | diag build | first try of the fix | after (a hosts from the character select): shown 1.29 s, `left`; play: shown 1.25 s; before (b hosts): hidden | 5 sessions, 4 clean, 1 cut (below) |
| `qcf-xrm08` | release build | the fix over repeated matches | a hosts from the character select (after, play, host-1, host-2): shown in 4 of 4 (1.26-1.28 s), after `left`; b hosts (before, join-1, join-2): hidden in 3 of 3 | 11 sessions, 10 clean, 1 cut (below) |
| `qcf-xrm09` | release build and the CPU pilot (Rematch Yes) | a later match, and the README takes | m0, m5 (rematch lobbies, m5 after a match that began in a character-select lobby with the plugin): shown 1.27 s / 1.26 s, `left`; m3 and ga (a hosts from the character select): shown, ga `left`; gb (b hosts): hidden | 4 sessions, 4 clean |

- **After the leave** (xrm07 and xrm08 `after`, xrm09 `ga`, xrm06 m0 and m5): a was at the character select, and b
  stayed in its lobby alone, searching again. Frames: `out/after/b-*-af-after.png`,
  `out/gif-after/b-*-ga-b-waits.png`, `out/m0-native/b-*-m0-end.png`.
- **Install line:**
  - xrm08 and xrm09 (next to the pilot): `plugin=quickplay-cancel-fix/0.2.0 wait=stock/near result=installed
    heap_allocations=0`, and nothing else.
  - xrm07 (diag) also logged `adopted slot=S count=1->2` once for each lobby a entered from the character select (4).
- **Both consoles:** the same sessions with the same end frames, and no `desync_seen` above 0 in any run. Every
  `console.log` and trace stream has no panic, abort, crash or unmapped-access line.
- **The fix in matches:**
  - As host: xrm07 play, xrm08 play, host-1 and host-2, xrm09 m3.
  - As joiner: xrm07 t1, xrm08 t1, join-1 and join-2, xrm09 t1 and gb.
  - All were played to the end, clean.

### README GIFs of 0.2.0 (replaced in 0.3.0; this `before.gif` ships with v0.2.1)

- Both clips come from `qcf-xrm09` (1280 wide, 10 fps), cut with `tools/gif_frames.py splice`.
  - Each is spliced from two takes of the same row on the console shown: the character select, then the lobby.
  - `before.gif` is b's screen (no QCF build): CSS take 12.0-13.0 s, then lobby take 4.0-12.5 s; 9.5 s in all.
  - `after.gif` is a's screen (release build): CSS take 12.0-13.0 s, then lobby take 4.0-9.4 s and 26.2-28.2 s.
    The cut is the load back to the character select. 8.4 s in all.
- The frames were checked one by one:
  - before: P2 "Joined!", then "Preparing…", and a menu with no Leave Battle for 6 s.
  - after: Leave Battle focused while P2 is "Preparing…", the dialog moving from No to Yes, the fade, and the
    character select.

### The cut match (xrm07 and xrm08 `before`)

- The `before` row recorded its GIF around the pause, so the match started inside the recording step. The runner
  encodes the take when the step ends, before the next step, and nothing pressed a button in the meantime.
- In xrm07:
  - The match started at 06:52:09. The take's capture ended at 06:52:13 and the encoded GIF was written at 06:52:59.
  - Both consoles' lockstep stopped at battle frame 2020: a 5 s gap, then no `sync_end`, the replay commit and the
    main menu. The battle step's first frame is 06:52:59.
- xrm08 shows the same at the same frame: an idle match plays out the same way each time.
- `join-1` and `join-2` in xrm08 repeat the row without the recording. b keeps the menu open through a's load and a
  joins with the plugin. Both matches were played and ended clean on both consoles.

### Online review (the one hook)

- **Where it runs:** `.1851674`, in PracticeScene's running state (the waiting area), after its update call. It
  returns to the stock instructions, so no game code is skipped or reordered. It reads the mode word, this console's
  member ID and the station records, then reads and writes one 32-bit count.
- **Random numbers:** none drawn, and none of the game's draws touched.
- **What it writes:**
  - Only the own slot's count in a Quickplay lobby (`0x5313fb4 + slot * 0xb0`), and only from 1 to 2.
  - A scan of the 13.0.5 code for the count's address finds the character select's reset (`.230d000`), the bump
    after a battle (`.2315f40`) and the waiting battle's member pass (`.15637a0`). A member's leave resets the slot
    through the slot table (`.16af2e0`).
  - The count is not part of a fighter, the stage or the round. The review didn't trace every user of the slot table
    (`0x5313f08`); the two-console matches below are the check on that.
- **Matches after it:** the matches after the change were played to the end on both consoles: the same sessions, the
  same end frames and `desync_seen=0` (xrm07, xrm08, xrm09, above).
- **Replays:** a replay never enters the waiting area, so the hook doesn't run there. No replay A/B was needed.

### Static checks

- **Sites:** five sites are checked against their 13.0.5 bytes before anything is installed.
  - Hooked: `.1851674` (PracticeScene's running state).
  - Relied on: `.230d000` (the reset), `.2315f40` (the bump), `.16f6340` (the station lookup) and `.1563f0c` (the mode
    word read).
  - The host tests derive the count table's address, the stride, the station byte and the mode word from those
    instructions (`the_relied_on_code_fixes_the_addresses`).
- **Card scan** (playbook `scan-offsets.py`, 2026-11-10) over the 20 plugins of the operator's card copy:
  - No plugin references the hook site, the reset, the bump or the count table.
  - TrainingRooms calls and fingerprints `.16f6340`, which the plugin only reads the layout of.
  - Five plugins read the online-mode word, as the plugin does.
