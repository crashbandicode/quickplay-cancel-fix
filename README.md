# quickplay-cancel-fix

> **Disclaimer:** this project was developed with the assistance of AI
> (Claude and other coding agents) under the operator's direction and
> review. Nothing is claimed beyond the dated tests cited below.

A Skyline plugin for Super Smash Bros. Ultimate 13.0.5. It gives you a
chance to leave a Quickplay match before it starts, from either side of the
room.

- **As the host** (you wait, an opponent joins): in the lobby you enter from
  the character select, the game doesn't let you leave while the opponent is
  loading in. The lobby after a "Rematch?" Yes does: Pause, Leave Battle,
  Yes. With this plugin the lobby from the character select behaves like the
  rematch lobby.
  - The game keeps a count of the battles played per player in the room. It
    offers the leave only when your own count is 2 or more, and resets the
    count to 1 when you come from the character select.
  - The plugin sets your own count from 1 to 2 while you wait in that lobby,
    and puts the 1 back when the lobby ends. So after each battle your
    console counts every player exactly as your opponent's console does.
- **As the joiner** (since 0.3.0): when your loading screen ends, your
  console reports "Ready!" at once. If the host is already waiting, the room
  goes to "Get Ready" within a second or two, and from then on Pause doesn't
  open.
  - The plugin holds your "Ready!" for a few seconds (3 by default) when you
    join a Quickplay room. The opponent sees you as "Preparing…", and Pause
    has Leave Battle.
  - When the hold ends, the plugin reports "Ready!" through the game's own
    call, and the match starts as usual. Leaving ends the hold.
- **Names:** the game shows "???" for a player you haven't played yet. The
  plugin shows the opponent's name instead.

The Leave Battle tab, the dialog and the leave are the game's own.

**Before** (joining a room, hold off):

![Before: joining a room, Leave Battle is there briefly, but "Get Ready" closes the dialog and the match starts](docs/media/before.gif)

*Eden, 2026-11-10, run `qcf-xhd4` row `h0`, 0.3.0 with `hold_seconds=0`
(the hold off). The console has joined another console's room from the
character select. Pause, Leave Battle, and "Would you like to leave the
game?" come up. Both players are "Ready!", "Get Ready" closes the dialog,
and the match goes on.*

**After** (joining a room, the 3 s hold):

![After: joining a room, during the hold Pause has Leave Battle, Yes, and back to the character select](docs/media/after.gif)

*Eden, 2026-11-10, run `qcf-xhd4` row `h3l`, 0.3.0 release build
(`d69f11cf…`) with the default hold. The same join and the same presses.
The console stays "Preparing…", so the room never reaches "Get Ready". Yes
leaves, and the load back to the character select starts.*

- **Quickplay only:** it acts only in online Quickplay. Other online modes
  and offline play are left alone. As the host, nothing is held.
- **Guarded hooks:** four hooks: the lobby's frame, the lobby's name check,
  the "Ready!" report and the leave. It installs only on 13.0.5, with the
  hooked instructions and the game code it relies on unchanged. Another
  plugin's hook on the same instruction is chained onto.
- **What it changes:**
  - your own battle count, 1 to 2, while you wait in a lobby from the
    character select;
  - what the name check reads for an opponent you haven't played yet;
  - as the joiner, when your "Ready!" goes out: the same report, through the
    game's own call, a few seconds later.
  - No message of its own is sent and no input is made. No heap use, no
    thread.

**Tested** ([evidence](docs/EVIDENCE.md)): Eden, 2026-11-10, 13.0.5 on LAN
Online, two consoles (one with the plugin, one without). 0.2.1's host side
was also played on a Switch.

- **Joining, held 3, 6 and 10 s:** in every held lobby, Pause showed Leave
  Battle and the opponent's name 1.2–1.6 s after the waiting area appeared.
  - The host saw the joiner as "Preparing…" for the whole hold.
  - Yes went back to the character select (4 of 4, from the character select
    and after a rematch). The host saw "Later!" and kept searching.
  - With no leave, the match started after the hold, with no kick or time-out.
    That held at 10 s, and at 6 s against a host 72 s into its room, past its
    countdown.
- **Matches after a hold**, including a rematch after a held match, played
  to their end on both consoles with no desync (time-out, then the sudden
  death).
- **Hosting from the character select** (0.2.1): Leave Battle while the
  opponent loaded, Yes back to the character select. The rematch match after
  a first match played to its end (v0.2.0 froze the opponent there).

## Install

Copy `libquickplay_cancel_fix.nro` to
`sd:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/`. The
install line goes to `sd:/ultimate/quickplay-cancel-fix/events.log`.

## Settings

`sd:/ultimate/quickplay-cancel-fix/config.ini` (optional):

```ini
hold_seconds=3
```

- How long a joined Quickplay match holds your "Ready!": 0.5 to 10 seconds
  (values outside are clamped). `0` turns the hold off.
- Without the file, the hold is 3 s.
- The file is read each time you join a match, so a change applies to the
  next one.

## Known limits

- **The hold runs while the menu is open.** With the default 3 s, Pause, Up,
  A, Right, A has to be done within about 3 s of the waiting area appearing.
  If that's too tight, raise `hold_seconds`; 5 is comfortable.
- **A hold always ends:** after the configured time, a started round or a
  leave, or after 10 s in which the menu couldn't open. So the host never
  waits on it for more than the hold plus 10 s.
- **The loading screen itself can't be left.** The game opens the menu only
  once the waiting area is up, and a press during the black loading screen
  isn't remembered.
- **The install line** ends `other_hook=member_test`. That's the plugin's own
  name hook overlapping a code check, not another plugin.
- Tested in Eden over LAN. The hold itself hasn't been played on a Switch yet.

## Build

```text
cargo test
cargo skyline build --release
```

The NRO is `target/aarch64-skyline-switch/release/libquickplay_cancel_fix.nro`.
`--features diag` builds the Eden diagnostic NRO (one log line per change).

## License

MIT
