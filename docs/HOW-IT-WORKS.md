# How it works

2026-11-13. Moved from the README and checked against `src/` at `bbe4661` (0.3.0). The measurements behind it are
in [EVIDENCE.md](EVIDENCE.md).

## As the host

- Quickplay keeps a count of the battles this console has played per member slot of the room
  (`0x5313fb4 + slot * 0xb0`). Entering the lobby from the character select resets every slot to 1 (`.230d000`);
  "Rematch?" Yes returns to the lobby without the reset. The game offers Leave Battle while an opponent loads only
  when this console's own count is 2 or more (EVIDENCE.md, "What decides the leave").
- The plugin writes 2 into its own slot when the slot holds 1 (`model::adopt`), while the waiting area runs, and puts
  the 1 back when the waiting area ends, on PracticeScene's exit request (`model::restore`, `model::frame`). After each
  battle both consoles then count every slot the same. v0.2.0 left the 2 in place, and that froze the opponent at the
  next rematch (EVIDENCE.md, 0.2.1).

## As the joiner (0.3.0)

- The waiting area's ready report is `bl .168db60` with 7 at `.151b044`. On every Quickplay match this console joins,
  the plugin reports 6 there instead, and calls the game's own `.168db60(svc, 7)` once `hold_seconds` of frames in
  which the waiting menu could open have passed (`src/hold.rs`).
- A started round or Leave Battle (`.1851930`) ends the hold, and so do 600 frames (10 s) in which the menu couldn't
  open. As the host nothing is held.

## Names

- The game shows "???" for a member whose slot count is below 2. The hook at `.1563ef8` makes that one load read 2
  for an opponent's slot (`model::shown`), so the name shows. No stored count changes.

## Hooks and guards

- Four hooks: the waiting area's frame (`.1851674`, `wait`), the name check (`.1563ef8`, `names`), the ready report
  (`.151b044`, `ready`) and the leave (`.1851930`, `leave`) (`model::HOOKED_SITES`).
- The game must read 13.0.5 (otherwise `result=unsupported_version`), and 13 sites are checked against their 13.0.5
  bytes before anything is installed (`model::SITES`: the four hooks and nine sites the plugin relies on).
- Another plugin's hook on a hooked instruction is chained onto (`src/chain.rs`).
- Install line in `qcf-xhd4`: `plugin=quickplay-cancel-fix/0.3.0 wait=stock/near names=stock/near leave=stock/near
  ready=stock/near hold_frames=180 other_hook=member_test result=installed heap_allocations=0`.

## What it changes

- Its own battle count, 1 to 2, while it waits in a lobby from the character select.
- What the name check reads for an opponent it hasn't played yet.
- As the joiner, when its "Ready!" goes out: the same report, through the game's own call, a few seconds later.
- No message of its own is sent and no input is made. No heap use (`heap_allocations=0`); the release build starts no
  thread (the writer thread is in the `diag` build only).

## Where it acts

- Everything checks the game's mode word (`0x53050f0`) for Quickplay's `0x07010101`. Other online modes, local
  wireless and offline play are left alone; an Eden local-wireless room logged `mode=0x02010000` (TrainingRooms' log in
  `a-ldn-fix020t-0c628724`, 2026-11-12). The plugin has not been run in local wireless.
