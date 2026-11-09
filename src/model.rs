//! Host-testable half: the hook site, the code it relies on, and the one
//! decision the hook makes.
//!
//! In an online room, the host's waiting battle runs `.1566650` every frame.
//! Once at least two members are in and every member reports state 7, it
//! starts the battle round (`.16a1df0`) either
//!
//! - at once, when the 20 s room countdown has run out (`.15637a0` returned 1
//!   into w20) in an online mode, or
//! - after the room has been full for 180 frames (the counter at waiting
//!   battle `+0x180`, `.15669dc..1566a00`).
//!
//! While no round has started, either player can still leave the waiting
//! area (PracticeScene's leave state `.18518b0` refuses only while the
//! round's ack mask is set). A joiner who arrives after the host's countdown
//! has run out is pulled into the round within a second, before it can leave;
//! one who arrives earlier gets the full-room wait. The plugin's only hook, at
//! the countdown test `.15669c4`, sends a full room down the full-room wait in
//! both cases. A room that is not full still starts on the countdown.

use crate::chain;

pub const SUPPORTED_VERSION: &str = "13.0.5";

/// How a site is checked before anything is installed. Other plugins' hooks
/// are accepted as `chain` reads them; any other change is not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// The plugin's inline hook goes here: another plugin's hook on it is
    /// chained onto (`chain::entry`).
    Hooked,
    /// Code the plugin calls, or relies on running as in 13.0.5: other
    /// plugins' hooks anywhere in it are fine to run through.
    Called,
}

/// A code site and the 64 bytes there in the 13.0.5 image.
pub struct Site {
    pub name: &'static str,
    pub offset: usize,
    pub kind: Kind,
    pub guard: [u8; 64],
}

impl Site {
    /// The 13.0.5 instructions.
    pub fn stock(&self) -> [u32; chain::WORDS] {
        chain::words(&self.guard)
    }

    /// Whether the plugin can use this site as `words` (at `address`) hold
    /// it: for the hooked site, whether its hook can go there.
    pub fn usable(&self, words: &[u32; chain::WORDS], address: u64) -> bool {
        let stock = self.stock();
        match self.kind {
            Kind::Hooked => chain::entry(&stock, words, address) != chain::Entry::Foreign,
            Kind::Called => chain::usable(&stock, words, address, 0),
        }
    }
}

/// `tbz w20, #0, .15669dc`: the countdown test, reached once at least two
/// members are in and all report state 7. w20 bit 0 = the countdown ran out,
/// x19 = the waiting battle (`+0x180` = full-room frames). The 64 bytes run
/// through the online-mode test, the full-room check and its counter. Nothing
/// jumps into the 20 bytes from here (`tools/branch_into.py`).
pub const VERDICT_OFFSET: usize = 0x15669c4;
pub const VERDICT: Site = Site {
    name: "verdict",
    offset: VERDICT_OFFSET,
    kind: Kind::Hooked,
    guard: [
        0xd4, 0x00, 0x00, 0x36, 0xc8, 0xf2, 0x40, 0xb9, 0x08, 0x19, 0x08, 0x12, 0xe9, 0x0b, 0x08,
        0x32, 0x1f, 0x01, 0x09, 0x6b, 0x60, 0x01, 0x00, 0x54, 0x28, 0x2f, 0x46, 0xf9, 0x08, 0x05,
        0x40, 0xf9, 0x00, 0x21, 0x02, 0x91, 0xe2, 0x42, 0x05, 0x94, 0x40, 0x09, 0x00, 0x36, 0x68,
        0x82, 0x41, 0xb9, 0x09, 0x05, 0x00, 0x11, 0x1f, 0xc9, 0x02, 0x71, 0x69, 0x82, 0x01, 0xb9,
        0x2d, 0xf7, 0xff, 0x54,
    ],
};

// The hook and the words it replaces are inside `.1566650` (which ends at
// `.1566be0`), before the full-room check it sends a full room to.
const _: () = assert!(0x1566650 < VERDICT_OFFSET && VERDICT_OFFSET + 20 <= 0x15669dc);

/// `.1566650`'s start, through `mov w20, w0` (`.1566688`): w20 is the
/// countdown's result (`.15637a0(x19, 0)`), kept in w20 to the hooked test.
pub const HOST_CHECK_OFFSET: usize = 0x1566650;
pub const HOST_CHECK: Site = Site {
    name: "host_check",
    offset: HOST_CHECK_OFFSET,
    kind: Kind::Called,
    guard: [
        0xff, 0x83, 0x02, 0xd1, 0xfc, 0x6f, 0x04, 0xa9, 0xfa, 0x67, 0x05, 0xa9, 0xf8, 0x5f, 0x06,
        0xa9, 0xf6, 0x57, 0x07, 0xa9, 0xf4, 0x4f, 0x08, 0xa9, 0xfd, 0x7b, 0x09, 0xa9, 0xfd, 0x43,
        0x02, 0x91, 0xe1, 0x03, 0x1f, 0x2a, 0xf3, 0x03, 0x00, 0xaa, 0x4a, 0xf4, 0xff, 0x97, 0x78,
        0xed, 0x01, 0xd0, 0x18, 0x63, 0x13, 0x91, 0x08, 0xff, 0xdf, 0x08, 0xf4, 0x03, 0x00, 0x2a,
        0x88, 0x24, 0x00, 0x36,
    ],
};

/// The full-room check `(service slot) -> bool` the host check calls at
/// `.15669e8`: members at least the room's size, nobody leaving. The hook
/// calls it with the same argument; it only reads.
pub const ROOM_FULL_OFFSET: usize = 0x16b7570;
pub const ROOM_FULL: Site = Site {
    name: "room_full",
    offset: ROOM_FULL_OFFSET,
    kind: Kind::Called,
    guard: [
        0xff, 0xc3, 0x01, 0xd1, 0xf7, 0x1b, 0x00, 0xf9, 0xf6, 0x57, 0x04, 0xa9, 0xf4, 0x4f, 0x05,
        0xa9, 0xfd, 0x7b, 0x06, 0xa9, 0xfd, 0x83, 0x01, 0x91, 0x16, 0x00, 0x40, 0xf9, 0x08, 0x02,
        0x94, 0x52, 0x75, 0xe0, 0x01, 0xb0, 0xd4, 0x02, 0x08, 0x8b, 0xa8, 0x2e, 0x46, 0xf9, 0x97,
        0x02, 0x40, 0xf9, 0x08, 0x05, 0x40, 0xf9, 0x08, 0x45, 0x40, 0xf9, 0x08, 0x75, 0x40, 0xb9,
        0xf3, 0x03, 0x00, 0xaa,
    ],
};

/// Every site checked before anything is installed.
pub const SITES: [&Site; 3] = [&VERDICT, &HOST_CHECK, &ROOM_FULL];

/// The main-image offsets this plugin hooks.
pub const HOOKED_SITES: [usize; 1] = [VERDICT_OFFSET];

/// `*(*(base + SERVICE) + 8) + SERVICE_SLOT` is the full-room check's
/// argument, as `.15669dc..15669e4` builds it; the online service is the
/// pointer stored there.
pub const SERVICE: usize = 0x52c4c58;
pub const SERVICE_SLOT: usize = 0x88;
/// The waiting battle's full-room frame counter (x19 + this).
pub const FULL_ROOM_FRAMES: usize = 0x180;
/// The round starts on the frame the counter, read before its increment, is
/// above 178 (`.15669f8`): the 180th full frame.
pub const FULL_ROOM_WAIT: u32 = 180;

/// The countdown result the hook leaves in w20: a full room takes the
/// full-room wait whether or not the countdown has run out.
pub fn verdict(countdown_out: bool, room_full: bool) -> bool {
    countdown_out && !room_full
}

/// What `.1566650` does on a frame once at least two members are in and
/// every member reports state 7.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    /// Starts the battle round.
    Start,
    /// Counts one more full-room frame (and starts after 180).
    Count,
    /// Resets the full-room counter: the room is not full.
    Reset,
}

/// `.15669c4..1566a00` as the game runs it, from the countdown result in w20,
/// the online-mode test and the full-room check, with the counter `frames`.
pub fn host_step(countdown_out: bool, online: bool, room_full: bool, frames: u32) -> Step {
    if countdown_out && online {
        Step::Start
    } else if !room_full {
        Step::Reset
    } else if frames + 1 >= FULL_ROOM_WAIT {
        Step::Start
    } else {
        Step::Count
    }
}

/// How a member's arrival plays out on the host: frames from the frame the
/// room is full and every member reports state 7 to the round start, with
/// the countdown `out` (or not) and `tick` = the frames to the countdown's
/// next result (it reports every 60 frames). `fixed` = with this plugin.
pub fn frames_to_round(countdown_out: bool, tick: u32, fixed: bool) -> u32 {
    let mut frames = 0;
    for frame in 0.. {
        let reported = countdown_out && frame % 60 == tick % 60;
        let w20 = if fixed { verdict(reported, true) } else { reported };
        match host_step(w20, true, true, frames) {
            Step::Start => return frame,
            Step::Count => frames += 1,
            Step::Reset => frames = 0,
        }
    }
    unreachable!()
}

/// A text line built in a fixed buffer; anything past `N` bytes is dropped.
pub struct Line<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> Line<N> {
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// The text plus a NUL, for `nn::fs` paths; `None` when there is no room.
    pub fn with_nul(&mut self) -> Option<&[u8]> {
        if self.len >= N {
            return None;
        }
        self.buf[self.len] = 0;
        Some(&self.buf[..=self.len])
    }
}

impl<const N: usize> Default for Line<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> core::fmt::Write for Line<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let take = s.len().min(N - self.len);
        self.buf[self.len..self.len + take].copy_from_slice(&s.as_bytes()[..take]);
        self.len += take;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Write as _;

    #[test]
    fn supported_version_is_13_0_5() {
        assert_eq!(SUPPORTED_VERSION, "13.0.5");
    }

    #[test]
    fn guards_start_with_the_expected_instructions() {
        let word = |site: &Site, i: usize| site.stock()[i];
        // tbz w20, #0, .15669dc
        assert_eq!(word(&VERDICT, 0), 0x3600_00d4);
        let imm14 = (word(&VERDICT, 0) >> 5) & 0x3fff;
        assert_eq!(VERDICT_OFFSET + imm14 as usize * 4, 0x15669dc);
        // b.eq .1566a04 (the online-mode start), then bl .16b7570 at .15669e8
        let b_eq = word(&VERDICT, 5);
        assert_eq!(b_eq & 0xff00_001f, 0x5400_0000);
        assert_eq!(VERDICT_OFFSET + 20 + ((b_eq >> 5) & 0x7ffff) as usize * 4, 0x1566a04);
        let bl = word(&VERDICT, 9);
        assert_eq!(bl >> 26, 0x25);
        assert_eq!(VERDICT_OFFSET + 36 + (bl & 0x03ff_ffff) as usize * 4, ROOM_FULL_OFFSET);
        // ldr w8, [x19, #0x180]; cmp w8, #178
        assert_eq!(word(&VERDICT, 11), 0xb941_8268);
        assert_eq!(word(&VERDICT, 13), 0x7102_c91f);
        assert_eq!((word(&VERDICT, 13) >> 10) & 0xfff, FULL_ROOM_WAIT - 2);
        // mov w1, wzr; mov x19, x0; bl .15637a0; ...; mov w20, w0
        assert_eq!(word(&HOST_CHECK, 8), 0x2a1f_03e1);
        let bl = word(&HOST_CHECK, 10);
        assert_eq!(
            (HOST_CHECK_OFFSET as i64 + 40 + (((bl & 0x03ff_ffff) as i32) << 6 >> 4) as i64) as usize,
            0x15637a0
        );
        assert_eq!(word(&HOST_CHECK, 14), 0x2a00_03f4);
        // sub sp, sp, #0x70
        assert_eq!(word(&ROOM_FULL, 0), 0xd101_c3ff);
    }

    /// Where each site would be with the image at a page-aligned base.
    fn at(site: &Site) -> u64 {
        0x80_0000_0000 + site.offset as u64
    }

    #[test]
    fn sites_are_checked_by_kind() {
        assert_eq!(VERDICT.kind, Kind::Hooked);
        assert_eq!(HOST_CHECK.kind, Kind::Called);
        assert_eq!(ROOM_FULL.kind, Kind::Called);
        assert_eq!(HOOKED_SITES, [VERDICT_OFFSET]);
        for site in SITES {
            assert!(site.usable(&site.stock(), at(site)), "{}", site.name);
        }
    }

    #[test]
    fn other_plugins_hooks_are_accepted_and_changes_are_not() {
        for site in SITES {
            // Another plugin's one-word branch over the first instruction.
            let mut words = site.stock();
            words[0] = 0x1400_0100;
            assert!(site.usable(&words, at(site)), "{}", site.name);
            // Anything that is not a hook.
            let mut words = site.stock();
            words[3] ^= 0x10;
            assert!(!site.usable(&words, at(site)), "{}", site.name);
        }
    }

    #[test]
    fn a_full_room_takes_the_full_room_wait() {
        // (countdown out, room full) -> the result the hook leaves.
        assert!(!verdict(false, false));
        assert!(!verdict(false, true));
        assert!(!verdict(true, true));
        assert!(verdict(true, false));
    }

    #[test]
    fn a_room_that_is_not_full_still_starts_on_the_countdown() {
        for online in [false, true] {
            for frames in [0, 10, 179] {
                let native = host_step(true, online, false, frames);
                let fixed = host_step(verdict(true, false), online, false, frames);
                assert_eq!(native, fixed, "online={online} frames={frames}");
            }
        }
        assert_eq!(host_step(true, true, false, 0), Step::Start);
    }

    #[test]
    fn offline_rooms_never_start_on_the_countdown() {
        // Local wireless rooms fail the online-mode test (`.15669c8..15669d8`).
        for countdown_out in [false, true] {
            for fixed in [false, true] {
                let w20 = if fixed {
                    verdict(countdown_out, true)
                } else {
                    countdown_out
                };
                assert_eq!(host_step(w20, false, true, 0), Step::Count);
                assert_eq!(host_step(w20, false, false, 0), Step::Reset);
            }
        }
    }

    #[test]
    fn crate_is_quickplay_cancel_fix() {
        assert_eq!(env!("CARGO_PKG_NAME"), "quickplay-cancel-fix");
        assert_eq!(env!("CARGO_PKG_VERSION"), "0.1.0");
    }

    #[test]
    fn host_step_follows_the_game() {
        assert_eq!(host_step(true, true, true, 0), Step::Start);
        assert_eq!(host_step(true, false, true, 0), Step::Count);
        assert_eq!(host_step(false, true, true, 0), Step::Count);
        assert_eq!(host_step(false, true, true, 178), Step::Count);
        assert_eq!(host_step(false, true, true, 179), Step::Start);
        assert_eq!(host_step(false, true, false, 179), Step::Reset);
    }

    /// The parity table: the arrival the game already leaves a leave window
    /// for (the host's countdown still running) and the one it does not (run
    /// out), on every countdown phase, with and without the plugin.
    #[test]
    fn a_late_arrival_gets_the_same_wait_as_an_early_one() {
        for tick in 0..60 {
            let early_native = frames_to_round(false, tick, false);
            let early_fixed = frames_to_round(false, tick, true);
            let late_native = frames_to_round(true, tick, false);
            let late_fixed = frames_to_round(true, tick, true);
            assert_eq!(early_native, FULL_ROOM_WAIT - 1, "tick {tick}");
            assert_eq!(early_fixed, early_native, "tick {tick}");
            assert_eq!(late_fixed, early_native, "tick {tick}");
            // Without the plugin the late arrival starts on the next report.
            assert_eq!(late_native, tick % 60, "tick {tick}");
        }
    }

    #[test]
    fn line_drops_overflow_and_adds_nul() {
        let mut line = Line::<8>::new();
        let _ = write!(line, "0123456789");
        assert_eq!(line.as_bytes(), b"01234567");
        assert!(line.with_nul().is_none());
        line.clear();
        let _ = write!(line, "ab");
        assert_eq!(line.with_nul(), Some(&b"ab\0"[..]));
    }
}
