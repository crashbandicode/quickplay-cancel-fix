# quickplay-cancel-fix

> **Disclaimer:** this project was developed with the assistance of AI
> (Claude and other coding agents) under the operator's direction and
> review. Nothing is claimed beyond the dated tests cited below.

A Skyline plugin for Super Smash Bros. Ultimate 13.0.5. It gives you a
chance to leave a Quickplay match before it starts, from either side of the
room: as the host while the opponent loads in, and as the joiner for a few
seconds after you arrive. The Leave Battle tab, the dialog and the leave are
the game's own.

**Before** (joining a room, no plugin):

<video src="https://git.syntaxbyexample.com/intpa/quickplay-cancel-fix/media/blob/afdedeaf3c93d92101baf87bf3a566322c267ed5" aria-label="Before: joining a room without the plugin, &quot;Get Ready&quot; at once, Pause opens nothing, and the match starts" autoplay muted loop playsinline controls></video> [Open the clip](docs/media/before.mp4)

*Without the plugin, Get Ready closes the leave window.*

**After** (joining a room, the 3 s hold):

<video src="https://git.syntaxbyexample.com/intpa/quickplay-cancel-fix/media/blob/2eecf5fafd4d6eb9a2435f9725c7aa724b8e0ec1" aria-label="After: joining a room, during the hold Pause has Leave Battle, Yes, and back to the character select" autoplay muted loop playsinline controls></video> [Open the clip](docs/media/after.mp4)

*With the hold, Pause opens Leave Battle.*

## What it does

- **As the host** (you wait, an opponent joins): the lobby you enter from the
  character select lets you leave while the opponent loads in (Pause, Leave
  Battle, Yes), as the lobby after a "Rematch?" Yes already does.
- **As the joiner** (since 0.3.0): your console says "Ready!" a few seconds
  later than usual (3 by default). Meanwhile the opponent sees you as
  "Preparing…" and Pause has Leave Battle. Then the match starts as usual.
- **Names:** the opponent's name shows instead of "???".
- **Quickplay only:** it acts in online Quickplay, Elite Smash included
  (since 0.3.1). Local wireless, other online modes and offline play are
  left alone.

How it does this, and what it changes in the game:
[docs/HOW-IT-WORKS.md](docs/HOW-IT-WORKS.md).

## Current release

Quickplay Cancel Fix **v0.3.1** (2026-11-23) is a release for Elite Smash players who want to identify opponents and leave before a match starts.

### What you need

| Thing | Upstream | Our fork | What the fork changes |
| --- | --- | --- | --- |
| Switch firmware | [23.5.0](https://en-americas-support.nintendo.com/app/answers/detail/a_id/22525) | — | — |
| Atmosphère | [1.12.2](https://github.com/Atmosphere-NX/Atmosphere/releases/latest) | [Atmosphère p1](https://git.syntaxbyexample.com/intpa/atmosphere-ro-cleanup-p1) | Cleans up after a failed module load. |
| Super Smash Bros. Ultimate | 13.0.5 | — | — |
| Skyline | [latest](https://github.com/skyline-dev/skyline/releases/latest) | [Skyline fork](https://git.syntaxbyexample.com/intpa/skyline) | Protects the logger queue from simultaneous access. |


### Install

1. Download [libquickplay_cancel_fix.nro](https://git.syntaxbyexample.com/intpa/quickplay-cancel-fix/releases/tag/v0.3.1). Close Smash.
2. Copy it to `sd:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libquickplay_cancel_fix.nro`. The two GIFs, `manifest.json` and `SHA256SUMS.txt` stay on your computer.
3. For a five-second hold, set `hold_seconds=5` in `sd:/ultimate/quickplay-cancel-fix/config.ini`. Without the file, the hold lasts three seconds.

**Update** — Replace the existing `libquickplay_cancel_fix.nro` in that folder. Keep your optional `config.ini`.

**Uninstall** — Delete `sd:/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libquickplay_cancel_fix.nro`.

### What changed

| Change | Guide |
| --- | --- |
| Show Elite Smash opponents’ names and icons in the waiting area. | [Quickplay guide](https://git.syntaxbyexample.com/intpa/quickplay-cancel-fix/src/tag/v0.3.1/docs/HOW-IT-WORKS.md) |
| Use Leave Battle while hosting in Elite Smash. | [Quickplay guide](https://git.syntaxbyexample.com/intpa/quickplay-cancel-fix/src/tag/v0.3.1/docs/HOW-IT-WORKS.md) |
| Get a few seconds to leave after joining an Elite Smash room. | [Quickplay guide](https://git.syntaxbyexample.com/intpa/quickplay-cancel-fix/src/tag/v0.3.1/docs/HOW-IT-WORKS.md) |

### Tested on

| Feature | Hardware | Eden |
| --- | --- | --- |
| Show Elite Smash opponents’ names and icons in the waiting area. | Yes | No |
| Use Leave Battle while hosting in Elite Smash. | Yes | No |
| Get a few seconds to leave after joining an Elite Smash room. | Yes | No |

### Known limits

- Eden testing covered plain Quickplay. The hold keeps counting down while the menu is open. Set `hold_seconds=5` for more time.
- You can leave from the waiting area only. Inputs during the black loading screen are discarded. [Details](https://git.syntaxbyexample.com/intpa/quickplay-cancel-fix/src/tag/v0.3.1/README.md).

Previous releases: [v0.3.0](https://git.syntaxbyexample.com/intpa/quickplay-cancel-fix/releases/tag/v0.3.0) · [v0.2.1](https://git.syntaxbyexample.com/intpa/quickplay-cancel-fix/releases/tag/v0.2.1).

## Settings

`sd:/ultimate/quickplay-cancel-fix/config.ini` (optional):

```ini
hold_seconds=5
```

- How long a joined Quickplay match holds your "Ready!": 0.5 to 10 seconds
  (values outside are clamped). `0` turns the hold off.
- **Recommended: 5.** Without the file the hold is 3 s, which leaves about
  3 s for Pause, Up, A, Right, A (see Known limits).
- The file is read each time you join a match, so a change applies to the
  next one.

## Known limits

- **The hold runs while the menu is open.** With the default 3 s, Pause, Up,
  A, Right, A has to be done within about 3 s of the waiting area appearing.
  `hold_seconds=5` gives room for it.
- **A hold always ends:** after the configured time, a started round or a
  leave, or after 10 s in which the menu couldn't open. So the host never
  waits on it for more than the hold plus 10 s.
- **The loading screen itself can't be left.** The game opens the menu only
  once the waiting area is up, and a press during the black loading screen
  isn't remembered.
- **The install line** ends `other_hook=member_test`. That's the plugin's own
  name hook overlapping a code check, not another plugin.

## Build

```text
cargo test
cargo skyline build --release
```

The NRO is `target/aarch64-skyline-switch/release/libquickplay_cancel_fix.nro`.
`--features diag` builds the Eden diagnostic NRO (one log line per change).

## License

MIT
