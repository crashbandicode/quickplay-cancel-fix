# quickplay-cancel-fix

> **Disclaimer:** this project was developed with the assistance of AI
> (Claude and other coding agents) under the operator's direction and
> review. Nothing is claimed beyond the dated tests cited below.

A Skyline plugin for Super Smash Bros. Ultimate 13.0.5. In Quickplay, after
a "Rematch?" Yes you wait in the lobby again, and while a new opponent is
still loading in you can leave: Pause, Leave Battle, Yes, back to the
character select, and the opponent keeps searching. The lobby you enter from
the character select doesn't offer that: once someone joins, Pause has no
Leave Battle until the match starts. With this plugin the lobby from the
character select behaves like the rematch lobby. The game keeps a count of
the battles played per player in the room. It resets the count when you
come from the character select and keeps it on a rematch, and you can leave
during the opponent's loading only when your own count is 2 or more. The
plugin sets your own count from 1 to 2, which is what one battle and a
rematch leave there. The Leave Battle tab, the dialog and the leave are the
game's own.

**Before** (no plugin):

![Before: the opponent joins and Pause has no Leave Battle](docs/media/before.gif)

*Eden, 2026-11-10, run `qcf-xrm09`, the console without the plugin: from
the character select into its Quickplay lobby, an opponent joins
("Preparing…"), and Pause has Training Partners and Behavior but no Leave
Battle. The wait for the opponent is cut.*

**After** (with the plugin):

![After: Leave Battle while the opponent loads, back to the character select](docs/media/after.gif)

*Eden, 2026-11-10, run `qcf-xrm09`, 0.2.0 release build (`0b12208d…`): the
same route on the console with the plugin. Pause has Leave Battle while the
opponent is "Preparing…", Yes leaves, and the character select comes back.
The wait for the opponent and the 17 s load back are cut.*

- **One hook, guarded:** it installs only on 13.0.5 with the hooked
  instruction and the four pieces of game code it relies on unchanged.
  Another plugin's hook on the same instruction is chained onto.
- **Quickplay only:** it acts only in online Quickplay. Lobbies reached by a
  rematch, other online modes and offline play are left alone.
- **One value written:** your own slot's battle count, 1 to 2, in the
  lobby. Nothing is sent, no input is made, and the opponent's count isn't
  touched. No heap use, no thread.

**Tested** ([evidence](docs/EVIDENCE.md)): Eden, 2026-11-10, 13.0.5 on LAN
Online, two consoles (one with the plugin, one without):

- **Without the plugin:** the lobby from the character select had no Leave
  Battle while the opponent loaded, in every try.
- **With it:** Leave Battle was there 1.3 s after Pause in each lobby from
  the character select. Yes took that console to the character select, and
  the other console stayed in its lobby, searching again. This is the same as
  the game's own leave after a rematch, which was recorded in the same setup.
- **Lobbies after a rematch Yes** keep the game's own leave with the plugin
  installed, also after a match that began in a lobby from the character
  select.
- **Matches played after it:** with the plugin hosting or joining, both
  consoles' desync detectors logged the same sessions, with no desync.

## Install

Copy `libquickplay_cancel_fix.nro` to
`sd:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/`. The
install line goes to `sd:/ultimate/quickplay-cancel-fix/events.log`.

## Build

```text
cargo test
cargo skyline build --release
```

The NRO is `target/aarch64-skyline-switch/release/libquickplay_cancel_fix.nro`.
`--features diag` builds the Eden diagnostic NRO (one log line per change).

## License

MIT
