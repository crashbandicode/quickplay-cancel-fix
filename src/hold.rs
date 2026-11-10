//! The joiner's ready hold (0.3.0, design B revived for every joined match).
//!
//! In an online room the host starts the battle round (`.16a1df0`) once every
//! member reports state 7, and either the room countdown has run out (it then
//! starts within a second, on the countdown's next 60-frame report) or the
//! room has been full for 180 frames. A console that joins reports 6 and then
//! 7 on the frame its waiting area starts (a small task PracticeScene runs:
//! `.168db60(svc, 6)`, then `.168db60(svc, 7)` at `.151b044`). Against a host
//! past its countdown, the round follows within a second.
//!
//! On every Quickplay match this console joins (from the character select or
//! after a rematch), the plugin reports 6 at `.151b044` instead, and calls the
//! game's own `.168db60(svc, 7)` once the hold has counted its frames. Only
//! frames in which the waiting menu can open count. Leaving the waiting area
//! drops the hold. The host is untouched: it waits for a member below 7.

use crate::model::{Kind, Site, QUICKPLAY};

/// The waiting area's ready report: `bl .168db60` with x0 = the online service
/// and w1 = 7, in the task (vtable `0x506d4d8`, slot 2 `.151af50`) PracticeScene
/// runs on the frame its waiting battle starts. `.151b02c` jumps to `.151b04c`,
/// inside the words a far jump would replace: a one-word branch only.
pub const READY_OFFSET: usize = 0x151b044;
pub const READY: Site = Site {
    name: "ready",
    offset: READY_OFFSET,
    kind: Kind::HookedNearOnly,
    guard: [
        0xc7, 0xca, 0x05, 0x94, 0x00, 0x02, 0x00, 0x37, 0x68, 0xef, 0x01, 0xb0, 0x08, 0x69, 0x87,
        0xb9, 0x1f, 0x05, 0x00, 0x71, 0x8b, 0x01, 0x00, 0x54, 0xe9, 0x03, 0x1f, 0xaa, 0xea, 0x0b,
        0x00, 0x32, 0x6b, 0xef, 0x01, 0xb0, 0x6b, 0x71, 0x1e, 0x91, 0x3f, 0x21, 0x00, 0xf1, 0x22,
        0x03, 0x00, 0x54, 0x29, 0x05, 0x00, 0x91, 0x6a, 0x01, 0x00, 0xb9, 0x6b, 0x21, 0x07, 0x91,
        0x3f, 0x01, 0x08, 0xeb,
    ],
};

// A far jump at the ready report would be 5 words (4 past an 8-byte boundary), over `.151b04c`.
const _: () = assert!(READY_OFFSET % 8 == 4);

/// PracticeScene's leave state (`.18518b0`, Leave Battle confirmed) on its
/// first frame, where it reads the round's ack mask to leave (0) or go back to
/// the waiting area. Nothing jumps into the 16 bytes from here; no flags live.
pub const LEAVE_OFFSET: usize = 0x1851930;
pub const LEAVE: Site = Site {
    name: "leave",
    offset: LEAVE_OFFSET,
    kind: Kind::Hooked,
    guard: [
        0x88, 0xd3, 0x01, 0xf0, 0x08, 0x2d, 0x46, 0xf9, 0x08, 0x05, 0x40, 0xf9, 0x08, 0x45, 0x40,
        0xf9, 0x89, 0x46, 0x8d, 0x52, 0x49, 0x00, 0xa0, 0x72, 0x08, 0x69, 0x69, 0xb8, 0x48, 0x01,
        0x00, 0x34, 0x68, 0xfe, 0x81, 0xb9, 0x1f, 0x4d, 0x00, 0x71, 0x42, 0x01, 0x00, 0x54, 0x68,
        0x0a, 0x08, 0x8b, 0xe9, 0x03, 0x00, 0x32, 0x09, 0xb1, 0x01, 0xb9, 0xfd, 0x7b, 0x41, 0xa9,
        0xf4, 0x4f, 0xc2, 0xa8,
    ],
};

/// The member-state setter `(svc, state) -> bool` the ready report calls; the
/// plugin calls it with 7 when the hold ends.
pub const SET_STATE_OFFSET: usize = 0x168db60;
pub const SET_STATE: Site = Site {
    name: "set_state",
    offset: SET_STATE_OFFSET,
    kind: Kind::Relied,
    guard: [
        0xfc, 0x6f, 0xba, 0xa9, 0xfa, 0x67, 0x01, 0xa9, 0xf8, 0x5f, 0x02, 0xa9, 0xf6, 0x57, 0x03,
        0xa9, 0xf4, 0x4f, 0x04, 0xa9, 0xfd, 0x7b, 0x05, 0xa9, 0xfd, 0x43, 0x01, 0x91, 0xff, 0x83,
        0x08, 0xd1, 0x08, 0xff, 0x93, 0x52, 0x18, 0x00, 0x08, 0x8b, 0x08, 0x38, 0x8d, 0x52, 0x48,
        0x00, 0xa0, 0x72, 0xbc, 0xe1, 0x01, 0xf0, 0x1b, 0x00, 0x08, 0x8b, 0x88, 0x2f, 0x46, 0xf9,
        0x75, 0x03, 0x40, 0xb9,
    ],
};

/// `() -> bool`: no screen fade, pop-up or dialog is up (the first half of
/// PracticeScene's menu gate `.18512c0`). Called, never hooked.
pub const INPUT_FREE_OFFSET: usize = 0x17f3810;
pub const INPUT_FREE: Site = Site {
    name: "input_free",
    offset: INPUT_FREE_OFFSET,
    kind: Kind::Relied,
    guard: [
        0xf3, 0x0f, 0x1e, 0xf8, 0xfd, 0x7b, 0x01, 0xa9, 0xfd, 0x43, 0x00, 0x91, 0xe8, 0xd9, 0x01,
        0x90, 0x08, 0x11, 0x44, 0xf9, 0xa8, 0x00, 0x00, 0xb4, 0x08, 0x05, 0x40, 0xf9, 0x08, 0x89,
        0x40, 0xf9, 0x00, 0x41, 0x40, 0xf9, 0x7f, 0xf1, 0x1b, 0x94, 0xe8, 0xd9, 0x01, 0x90, 0x08,
        0x15, 0x44, 0xf9, 0xa8, 0x01, 0x00, 0xb4, 0x13, 0x05, 0x40, 0xf9, 0x68, 0x8a, 0x40, 0xf9,
        0x00, 0x41, 0x40, 0xf9,
    ],
};

/// PracticeScene's menu gate `(scene) -> bool` (`.18512c0`): `.17f3810()` and
/// the context's `+0x128` equal to the global at `MENU_FOCUS`. The plugin
/// evaluates it from the context, so its 64 bytes are checked as relied on.
pub const MENU_GATE_OFFSET: usize = 0x18512c0;
pub const MENU_GATE: Site = Site {
    name: "menu_gate",
    offset: MENU_GATE_OFFSET,
    kind: Kind::Relied,
    guard: [
        0xf3, 0x0f, 0x1e, 0xf8, 0xfd, 0x7b, 0x01, 0xa9, 0xfd, 0x43, 0x00, 0x91, 0xf3, 0x03, 0x00,
        0xaa, 0x50, 0x89, 0xfe, 0x97, 0x00, 0x01, 0x00, 0x36, 0x68, 0x2e, 0x40, 0xf9, 0x89, 0xd3,
        0x01, 0xd0, 0x29, 0xc5, 0x43, 0xf9, 0x08, 0x95, 0x40, 0xf9, 0x1f, 0x01, 0x09, 0xeb, 0xe0,
        0x17, 0x9f, 0x1a, 0x02, 0x00, 0x00, 0x14, 0xe0, 0x03, 0x1f, 0x2a, 0xfd, 0x7b, 0x41, 0xa9,
        0xf3, 0x07, 0x42, 0xf8,
    ],
};
/// The global the menu gate compares with the context's `+0x128`.
pub const MENU_FOCUS: usize = 0x52c3788;
pub const PRACTICE_FOCUS: usize = 0x128;

/// The service's room host ID (this console's own is `model::SVC_MY_ID`) and
/// the round's ack mask (non-zero once the host started the round).
pub const SVC_HOST_ID: usize = 0x9ff0;
pub const SVC_ROUND_MASK: usize = 0x26a34;

pub const READY_STATE: u32 = 7;
pub const LOADING_STATE: u32 = 6;

/// The host's full-room wait: the round starts on the 180th full frame
/// (`.1564fd8..1564fe8`: the counter read before its increment is above 178).
pub const FULL_ROOM_WAIT: u32 = 180;

/// The hold, in frames (60 per second): the default, and the bounds a config
/// value is clamped to.
pub const DEFAULT_HOLD: u32 = 180;
pub const MIN_HOLD: u32 = 30;
pub const MAX_HOLD: u32 = 600;
/// Frames in which the menu couldn't open (a fade, a pop-up, the leave
/// dialog) after which a hold ends anyway, so the host never waits on it for
/// more than the hold plus this.
pub const MAX_PAUSED: u32 = 600;

/// The menu gate's second half: the PracticeScene context's focus handle is
/// the current one.
pub fn practice_has_focus(context_focus: u64, current: u64) -> bool {
    context_focus != 0 && context_focus == current
}

/// The hold from `config.ini` text: `hold_seconds=<number>` (decimals allowed),
/// clamped to `MIN_HOLD..=MAX_HOLD` frames; `hold_seconds=0` turns the hold
/// off; anything missing or unreadable gives the default.
pub fn hold_from_config(text: &[u8]) -> u32 {
    let mut hold = DEFAULT_HOLD;
    for line in text.split(|b| *b == b'\n') {
        let Some(eq) = line.iter().position(|b| *b == b'=') else {
            continue;
        };
        if line[..eq].trim_ascii() != b"hold_seconds" {
            continue;
        }
        let value = core::str::from_utf8(line[eq + 1..].trim_ascii()).ok();
        hold = match value.and_then(|v| v.parse::<f32>().ok()) {
            Some(s) if s.is_finite() && s > 0.0 => ((s * 60.0).round() as u32).clamp(MIN_HOLD, MAX_HOLD),
            Some(s) if s.is_finite() && s.abs() < f32::EPSILON => 0,
            _ => DEFAULT_HOLD,
        };
    }
    hold
}

/// This console's place in the room.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    NoRoom,
    Host,
    Joiner,
}

/// From the service's member IDs: no room without an own ID; host when the
/// room's host ID is this console's.
pub fn role(my_id: u64, host_id: u64) -> Role {
    if my_id == 0 {
        Role::NoRoom
    } else if host_id == my_id {
        Role::Host
    } else {
        Role::Joiner
    }
}

/// What a frame of the wait loop does with the hold.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tick {
    /// No hold.
    Idle,
    /// Holding; frames counted so far.
    Holding(u32),
    /// Holding, but this frame doesn't count (the waiting menu couldn't open).
    Paused(u32),
    /// The hold ends: report 7 now.
    Release,
}

/// What happened to a hold, for the diagnostic log.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Note {
    HeldAsJoiner,
    NativeAsHost,
    NativeNoRoom,
    Released,
    ReleasedPaused,
    DroppedLeave,
    DroppedRound,
    DroppedStale,
}

/// The joiner's hold, driven by the ready report, the waiting-area frame and
/// the leave.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Gate {
    /// Frames to hold (0: off).
    limit: u32,
    /// Frames held so far, while holding.
    held: Option<u32>,
    /// Frames of this hold in which the menu couldn't open.
    paused: u32,
}

impl Gate {
    pub const fn new(limit: u32) -> Self {
        Self { limit, held: None, paused: 0 }
    }

    /// The hold for the next waiting area (the config is read per match).
    pub fn set_limit(&mut self, limit: u32) {
        self.limit = limit;
    }

    pub fn holding(&self) -> bool {
        self.held.is_some()
    }

    /// The waiting area is about to report `state` with this console in
    /// `role`, the online mode being `mode`. Returns the state to report and
    /// what happened.
    pub fn ready(&mut self, state: u32, role: Role, mode: u32) -> (u32, Option<Note>) {
        if state != READY_STATE {
            return (state, None);
        }
        // A new waiting area: a hold from an earlier one (left some other way) ends.
        let stale = self.held.take().map(|_| Note::DroppedStale);
        if mode != QUICKPLAY || self.limit == 0 {
            return (state, stale);
        }
        match role {
            Role::Joiner => {
                self.held = Some(0);
                self.paused = 0;
                (LOADING_STATE, Some(Note::HeldAsJoiner))
            }
            Role::Host => (state, stale.or(Some(Note::NativeAsHost))),
            Role::NoRoom => (state, stale.or(Some(Note::NativeNoRoom))),
        }
    }

    /// A waiting-area frame while holding; `round_started` = the ack mask is
    /// set; `input_free` = the waiting menu could open this frame. Only such
    /// frames count.
    pub fn tick(&mut self, round_started: bool, input_free: bool) -> (Tick, Option<Note>) {
        let Some(held) = self.held else {
            return (Tick::Idle, None);
        };
        if round_started {
            self.held = None;
            return (Tick::Idle, Some(Note::DroppedRound));
        }
        if !input_free {
            self.paused += 1;
            if self.paused >= MAX_PAUSED {
                self.held = None;
                return (Tick::Release, Some(Note::ReleasedPaused));
            }
            return (Tick::Paused(held), None);
        }
        let held = held + 1;
        if held >= self.limit {
            self.held = None;
            return (Tick::Release, Some(Note::Released));
        }
        self.held = Some(held);
        (Tick::Holding(held), None)
    }

    /// Leave Battle was confirmed in the waiting area: the hold ends.
    pub fn leave(&mut self) -> Option<Note> {
        self.held.take().map(|_| Note::DroppedLeave)
    }
}

/// What `.1564a30` does on a host frame once at least two members are in and
/// every member reports 7.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    Start,
    Count,
    Reset,
}

/// `.1564f4c..1564fe8` as the game runs it on the host: the countdown's result,
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

/// A vanilla host and a joiner in a full 1v1 room: frames from the joiner's
/// waiting area starting to the host starting the round, with the host's
/// countdown `out` or still running and `tick` = the frames to the countdown's
/// next report. `hold` = the joiner holds its 7 that many frames first. This
/// is the joiner's leave window.
pub fn leave_window(countdown_out: bool, tick: u32, hold: u32) -> u32 {
    let mut frames = 0;
    for frame in 0.. {
        if frame < hold {
            continue; // the host waits for a member below 7: nothing counts
        }
        let reported = countdown_out && frame % 60 == tick % 60;
        match host_step(reported, true, true, frames) {
            Step::Start => return frame,
            Step::Count => frames += 1,
            Step::Reset => frames = 0,
        }
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain;

    fn bl_target(site: &Site, i: usize) -> usize {
        let word = site.stock()[i];
        assert_eq!(word >> 26, 0x25, "{} word {i} is not a bl", site.name);
        let imm = ((word & 0x03ff_ffff) as i32) << 6 >> 4;
        (site.offset as i64 + 4 * i as i64 + imm as i64) as usize
    }

    fn at(site: &Site) -> u64 {
        0x80_0000_0000 + site.offset as u64
    }

    #[test]
    fn guards_start_with_the_expected_instructions() {
        // bl .168db60; tbnz w0, #0
        assert_eq!(bl_target(&READY, 0), SET_STATE_OFFSET);
        assert_eq!(READY.stock()[1], 0x3700_0200);
        // adrp x8, 0x52c4000; ldr x8, [x8, #0xc58]; ...; mov w9, #0x6a34; movk w9, #2, lsl #16 (the ack mask)
        assert_eq!(LEAVE.stock()[1], 0xf946_2d08);
        assert_eq!(LEAVE.stock()[4..6], [0x528d_4689, 0x72a0_0049]);
        assert_eq!(0x2_0000 | ((LEAVE.stock()[4] >> 5) & 0xffff) as usize, SVC_ROUND_MASK);
        // bl .17f3810; ...; ldr x8, [x19, #0x58]; adrp x9; ldr x9, [x9, #0x788]; ldr x8, [x8, #0x128]
        assert_eq!(bl_target(&MENU_GATE, 4), INPUT_FREE_OFFSET);
        assert_eq!(MENU_GATE.stock()[6], 0xf940_2e68);
        assert_eq!(MENU_GATE.stock()[8], 0xf943_c529);
        assert_eq!(MENU_GATE.stock()[9], 0xf940_9508);
        assert_eq!((MENU_GATE.stock()[8] >> 10 & 0xfff) as usize * 8, MENU_FOCUS & 0xfff);
        assert_eq!((MENU_GATE.stock()[9] >> 10 & 0xfff) as usize * 8, PRACTICE_FOCUS);
        assert_eq!(SET_STATE.stock()[0], 0xa9ba_6ffc);
        assert_eq!(INPUT_FREE.stock()[0], 0xf81e_0ff3);
        assert!(practice_has_focus(0x1234, 0x1234));
        assert!(!practice_has_focus(0x1234, 0x5678));
        assert!(!practice_has_focus(0, 0));
    }

    #[test]
    fn a_far_jump_over_the_ready_report_is_refused() {
        assert!(READY.usable(&READY.stock(), at(&READY)));
        let mut words = READY.stock();
        words[0] = 0x1400_0100; // another plugin's one-word branch: fine
        assert!(READY.usable(&words, at(&READY)));
        let mut words = READY.stock();
        words[0] = chain::NOP;
        words[1..3].copy_from_slice(&chain::FAR_JUMP);
        words[3] = 0x4567_8000;
        words[4] = 0x71;
        assert!(!READY.usable(&words, at(&READY)));
    }

    #[test]
    fn roles_come_from_the_member_ids() {
        assert_eq!(role(0, 0), Role::NoRoom);
        assert_eq!(role(0, 5), Role::NoRoom);
        assert_eq!(role(5, 5), Role::Host);
        assert_eq!(role(5, 9), Role::Joiner);
        assert_eq!(role(5, 0), Role::Joiner);
    }

    #[test]
    fn every_joined_quickplay_match_is_held_and_hosts_are_not() {
        let mut gate = Gate::new(DEFAULT_HOLD);
        // The first match from the character select, then rematches: each joined one is held.
        for _ in 0..3 {
            assert_eq!(gate.ready(READY_STATE, Role::Joiner, QUICKPLAY), (LOADING_STATE, Some(Note::HeldAsJoiner)));
            assert!(gate.holding());
            while gate.tick(false, true).0 != Tick::Release {}
            assert!(!gate.holding());
        }
        for (role, note) in [(Role::Host, Note::NativeAsHost), (Role::NoRoom, Note::NativeNoRoom)] {
            assert_eq!(gate.ready(READY_STATE, role, QUICKPLAY), (READY_STATE, Some(note)), "{role:?}");
            assert!(!gate.holding());
        }
        // Other modes, other states, and the hold turned off: native.
        for mode in [0, 0x0701_0102, 0x0701_0400, 0x0701_0800] {
            assert_eq!(gate.ready(READY_STATE, Role::Joiner, mode), (READY_STATE, None), "{mode:#x}");
        }
        assert_eq!(gate.ready(LOADING_STATE, Role::Joiner, QUICKPLAY), (LOADING_STATE, None));
        let mut off = Gate::new(0);
        assert_eq!(off.ready(READY_STATE, Role::Joiner, QUICKPLAY), (READY_STATE, None));
    }

    #[test]
    fn releases_after_the_configured_hold() {
        for limit in [MIN_HOLD, DEFAULT_HOLD, MAX_HOLD] {
            let mut gate = Gate::new(limit);
            gate.ready(READY_STATE, Role::Joiner, QUICKPLAY);
            for frame in 1..limit {
                assert_eq!(gate.tick(false, true), (Tick::Holding(frame), None));
            }
            assert_eq!(gate.tick(false, true), (Tick::Release, Some(Note::Released)));
            assert_eq!(gate.tick(false, true), (Tick::Idle, None));
        }
    }

    #[test]
    fn only_frames_with_the_menu_available_count() {
        let mut gate = Gate::new(DEFAULT_HOLD);
        gate.ready(READY_STATE, Role::Joiner, QUICKPLAY);
        // The waiting area fading in behind the loading tip: nothing counts.
        for _ in 0..300 {
            assert_eq!(gate.tick(false, false), (Tick::Paused(0), None));
        }
        for frame in 1..=100 {
            assert_eq!(gate.tick(false, true), (Tick::Holding(frame), None));
        }
        assert_eq!(gate.tick(false, false), (Tick::Paused(100), None));
        for frame in 101..DEFAULT_HOLD {
            assert_eq!(gate.tick(false, true), (Tick::Holding(frame), None));
        }
        assert_eq!(gate.tick(false, true), (Tick::Release, Some(Note::Released)));
    }

    /// A menu that never becomes available (or a leave dialog left open) doesn't keep the host waiting: the hold
    /// ends after `MAX_PAUSED` such frames, counted over the whole hold.
    #[test]
    fn the_host_never_waits_more_than_the_hold_and_the_pause_cap() {
        let mut gate = Gate::new(MAX_HOLD);
        gate.ready(READY_STATE, Role::Joiner, QUICKPLAY);
        for _ in 1..MAX_PAUSED {
            assert_eq!(gate.tick(false, false), (Tick::Paused(0), None));
        }
        assert_eq!(gate.tick(false, false), (Tick::Release, Some(Note::ReleasedPaused)));
        assert!(!gate.holding());
        // Mixed: the counted and paused frames together never pass the hold plus the cap.
        let mut gate = Gate::new(MAX_HOLD);
        gate.ready(READY_STATE, Role::Joiner, QUICKPLAY);
        let mut frames = 0;
        loop {
            frames += 1;
            if gate.tick(false, frames % 3 != 0).0 == Tick::Release {
                break;
            }
        }
        assert!(frames <= MAX_HOLD + MAX_PAUSED, "{frames}");
        // The next hold starts with no paused frames.
        gate.ready(READY_STATE, Role::Joiner, QUICKPLAY);
        assert_eq!(gate.tick(false, false), (Tick::Paused(0), None));
    }

    #[test]
    fn each_match_takes_the_hold_configured_then() {
        let mut gate = Gate::new(DEFAULT_HOLD);
        for limit in [hold_from_config(b"hold_seconds=6"), hold_from_config(b"hold_seconds=10"), MIN_HOLD] {
            gate.set_limit(limit);
            gate.ready(READY_STATE, Role::Joiner, QUICKPLAY);
            let mut frames = 1;
            while gate.tick(false, true).0 != Tick::Release {
                frames += 1;
            }
            assert_eq!(frames, limit);
        }
        gate.set_limit(hold_from_config(b"hold_seconds=0"));
        assert_eq!(gate.ready(READY_STATE, Role::Joiner, QUICKPLAY), (READY_STATE, None));
    }

    #[test]
    fn a_leave_or_a_started_round_drops_the_hold() {
        let mut gate = Gate::new(DEFAULT_HOLD);
        gate.ready(READY_STATE, Role::Joiner, QUICKPLAY);
        gate.tick(false, true);
        assert_eq!(gate.leave(), Some(Note::DroppedLeave));
        assert_eq!(gate.tick(false, true), (Tick::Idle, None));
        assert_eq!(gate.leave(), None);
        let mut gate = Gate::new(DEFAULT_HOLD);
        gate.ready(READY_STATE, Role::Joiner, QUICKPLAY);
        assert_eq!(gate.tick(true, false), (Tick::Idle, Some(Note::DroppedRound)));
        // The waiting area ended some other way: the next ready report ends the stale hold and holds anew.
        let mut gate = Gate::new(DEFAULT_HOLD);
        gate.ready(READY_STATE, Role::Host, QUICKPLAY);
        gate.ready(READY_STATE, Role::Joiner, QUICKPLAY);
        assert_eq!(gate.ready(READY_STATE, Role::Host, QUICKPLAY), (READY_STATE, Some(Note::DroppedStale)));
        assert!(!gate.holding());
    }

    #[test]
    fn the_hold_comes_from_the_config_file() {
        assert_eq!(hold_from_config(b""), DEFAULT_HOLD);
        assert_eq!(hold_from_config(b"hold_seconds=3\n"), 180);
        assert_eq!(hold_from_config(b"# comment\n hold_seconds = 6 \r\n"), 360);
        assert_eq!(hold_from_config(b"hold_seconds=2.5"), 150);
        assert_eq!(hold_from_config(b"hold_seconds=0"), 0);
        assert_eq!(hold_from_config(b"hold_seconds=0.1"), MIN_HOLD);
        assert_eq!(hold_from_config(b"hold_seconds=99"), MAX_HOLD);
        for bad in [&b"hold_seconds=abc"[..], b"hold_seconds=-3", b"hold_seconds=nan", b"hold_seconds=inf", b"other=4"] {
            assert_eq!(hold_from_config(bad), DEFAULT_HOLD, "{}", String::from_utf8_lossy(bad));
        }
    }

    #[test]
    fn host_step_follows_the_game() {
        assert_eq!(host_step(true, true, true, 0), Step::Start);
        assert_eq!(host_step(true, false, true, 0), Step::Count);
        assert_eq!(host_step(false, true, true, 178), Step::Count);
        assert_eq!(host_step(false, true, true, 179), Step::Start);
        assert_eq!(host_step(false, true, false, 179), Step::Reset);
        assert_eq!(FULL_ROOM_WAIT - 2, 178);
    }

    /// The joiner's leave window against a vanilla host, on every countdown
    /// phase: a late joiner (the host's countdown out) gets under a second
    /// natively and at least the hold with it.
    #[test]
    fn a_held_joiner_always_gets_a_window() {
        for tick in 0..60 {
            let late_native = leave_window(true, tick, 0);
            assert!(late_native < 60, "tick {tick}");
            for hold in [MIN_HOLD, DEFAULT_HOLD, MAX_HOLD] {
                let late_held = leave_window(true, tick, hold);
                assert!(late_held >= hold, "tick {tick} hold {hold}");
                assert!(late_held < hold + 60, "tick {tick} hold {hold}");
            }
        }
        // An early joiner (countdown running): the host's own full-room wait follows the hold.
        assert_eq!(leave_window(false, 0, 0), FULL_ROOM_WAIT - 1);
        assert_eq!(leave_window(false, 0, DEFAULT_HOLD), DEFAULT_HOLD + FULL_ROOM_WAIT - 1);
    }
}
