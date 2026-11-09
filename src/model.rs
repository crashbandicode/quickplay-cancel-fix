//! Host-testable half: the hook sites, the code they rely on, and the
//! first-match hold.
//!
//! In an online room the host starts the battle round (`.16a1df0`) once every
//! member reports state 7 and either the room countdown has run out (it then
//! starts within a second, on the countdown's next 60-frame report) or the
//! room has been full for 180 frames. Until the round starts either player can
//! leave the waiting area (PracticeScene refuses only while the round's ack
//! mask is set). A console that joins a room whose host has waited past its
//! countdown reports 7 on the frame its waiting area starts (a small task run
//! from PracticeScene: `.168db60(svc, 6)` then `.168db60(svc, 7)` at
//! `.151b044`), and the round follows within a second: no leave. One that
//! joins earlier gets the 180-frame full-room wait.
//!
//! For the first match of a Quickplay session (the character select's Ready
//! that starts the service's matchmaking mode), and only when this console
//! joins the room, the plugin reports 6 at `.151b044` instead and the game's
//! own `.168db60(svc, 7)` 180 waiting-area frames later: the same wait an
//! earlier arrival gets. The host sees a joiner that is still loading. Leaving
//! the waiting area drops the hold. Every later match is untouched.

use crate::chain;

pub const SUPPORTED_VERSION: &str = "13.0.5";

/// How a site is checked before anything is installed. Other plugins' hooks
/// are accepted as `chain` reads them; any other change is not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// One of the plugin's inline hooks goes here: another plugin's hook on it
    /// is chained onto (`chain::entry`).
    Hooked,
    /// Code the plugin calls: other plugins' hooks anywhere in it are fine to
    /// run through.
    Called,
}

/// A code site and the 64 bytes there in the 13.0.5 image.
pub struct Site {
    pub name: &'static str,
    pub offset: usize,
    pub kind: Kind,
    /// The hook may only be a one-word branch here: code jumps into the words
    /// a far jump would replace.
    pub near_only: bool,
    pub guard: [u8; 64],
}

impl Site {
    /// The 13.0.5 instructions.
    pub fn stock(&self) -> [u32; chain::WORDS] {
        chain::words(&self.guard)
    }

    /// Whether the plugin can use this site as `words` (at `address`) hold
    /// it: for a hooked site, whether its hook can go there.
    pub fn usable(&self, words: &[u32; chain::WORDS], address: u64) -> bool {
        let stock = self.stock();
        match self.kind {
            Kind::Hooked => match chain::entry(&stock, words, address) {
                chain::Entry::Foreign => false,
                // Another plugin's far jump already covers the words code jumps into.
                chain::Entry::Chain(_) => !self.near_only,
                _ => true,
            },
            Kind::Called => chain::usable(&stock, words, address, 0),
        }
    }
}

/// The Quickplay sequence's search start (its state 5 handler `.23692b0`, on
/// the state's first frame): `orr w1, wzr, #2` before `bl .16b4920(svc, 2)`,
/// x8 = the online service. The service's mode (`+0x74`) is still 0 when the
/// search starts a new Quickplay session from the character select; the call
/// returns at once when the session is already in that mode. Nothing jumps
/// into the 20 bytes from here. (TrainingRooms fingerprints the sequence's
/// frame `.236b470` and PracticeScene's `.1851480`; neither is touched.)
pub const SEARCH_OFFSET: usize = 0x2369304;
pub const SEARCH: Site = Site {
    name: "search",
    offset: SEARCH_OFFSET,
    kind: Kind::Hooked,
    near_only: false,
    guard: [
        0xe1, 0x03, 0x1f, 0x32, 0xe0, 0x03, 0x08, 0xaa, 0x85, 0x2d, 0xcd, 0x97, 0x88, 0x2e, 0x46,
        0xf9, 0x09, 0x21, 0x00, 0x91, 0x28, 0x01, 0x40, 0xf9, 0x08, 0x41, 0x40, 0xf9, 0x09, 0x11,
        0x49, 0x39, 0x89, 0x00, 0x00, 0x34, 0x08, 0x19, 0x42, 0xb9, 0x1f, 0x25, 0x00, 0x71, 0xc1,
        0x00, 0x00, 0x54, 0x60, 0x06, 0x40, 0xf9, 0xfd, 0x7b, 0x41, 0xa9, 0xe1, 0x03, 0x1f, 0x32,
        0xf4, 0x4f, 0xc2, 0xa8,
    ],
};

/// The waiting area's ready report: `bl .168db60` with x0 = the online
/// service and w1 = 7, in the task (vtable `0x506d4d8`, slot 2 `.151af50`)
/// PracticeScene runs on the frame its waiting battle starts. `.151b02c`
/// jumps to `.151b04c`, inside the words a far jump would replace: near only.
pub const READY_OFFSET: usize = 0x151b044;
pub const READY: Site = Site {
    name: "ready",
    offset: READY_OFFSET,
    kind: Kind::Hooked,
    near_only: true,
    guard: [
        0xc7, 0xca, 0x05, 0x94, 0x00, 0x02, 0x00, 0x37, 0x68, 0xef, 0x01, 0xb0, 0x08, 0x69, 0x87,
        0xb9, 0x1f, 0x05, 0x00, 0x71, 0x8b, 0x01, 0x00, 0x54, 0xe9, 0x03, 0x1f, 0xaa, 0xea, 0x0b,
        0x00, 0x32, 0x6b, 0xef, 0x01, 0xb0, 0x6b, 0x71, 0x1e, 0x91, 0x3f, 0x21, 0x00, 0xf1, 0x22,
        0x03, 0x00, 0x54, 0x29, 0x05, 0x00, 0x91, 0x6a, 0x01, 0x00, 0xb9, 0x6b, 0x21, 0x07, 0x91,
        0x3f, 0x01, 0x08, 0xeb,
    ],
};

// A far jump at the ready report would be 5 words (it is 4 past an 8-byte
// boundary), over `.151b04c`.
const _: () = assert!(READY.near_only && READY_OFFSET % 8 == 4);

/// The waiting battle's wait loop (`.1564a30`), the branch a console that is
/// not the room's host takes every frame: it reads the round's ack mask and
/// waits while it is 0. x19 = the waiting battle. Nothing jumps into the 16
/// bytes from here; no flags are live.
pub const WAIT_OFFSET: usize = 0x1564d78;
pub const WAIT: Site = Site {
    name: "wait",
    offset: WAIT_OFFSET,
    kind: Kind::Hooked,
    near_only: false,
    guard: [
        0x88, 0x2f, 0x46, 0xf9, 0x08, 0x05, 0x40, 0xf9, 0x08, 0x45, 0x40, 0xf9, 0x89, 0x46, 0x8d,
        0x52, 0x49, 0x00, 0xa0, 0x72, 0x08, 0x69, 0x69, 0xb8, 0xe8, 0x1d, 0x00, 0x34, 0x08, 0x00,
        0x00, 0xb0, 0x08, 0x01, 0x2c, 0x91, 0x5f, 0xff, 0xff, 0x17, 0x68, 0xdf, 0x43, 0xf9, 0x88,
        0x00, 0x00, 0xb4, 0x08, 0x05, 0x40, 0xf9, 0x08, 0x99, 0x40, 0xb9, 0xe8, 0xfa, 0xff, 0x35,
        0x48, 0xee, 0x01, 0xf0,
    ],
};

/// PracticeScene's leave state (`.18518b0`, Leave Battle confirmed) on its
/// first frame, where it reads the round's ack mask to leave (0) or go back to
/// the waiting area. x19 = the PracticeScene context. Nothing jumps into the
/// 16 bytes from here; no flags are live.
pub const LEAVE_OFFSET: usize = 0x1851930;
pub const LEAVE: Site = Site {
    name: "leave",
    offset: LEAVE_OFFSET,
    kind: Kind::Hooked,
    near_only: false,
    guard: [
        0x88, 0xd3, 0x01, 0xf0, 0x08, 0x2d, 0x46, 0xf9, 0x08, 0x05, 0x40, 0xf9, 0x08, 0x45, 0x40,
        0xf9, 0x89, 0x46, 0x8d, 0x52, 0x49, 0x00, 0xa0, 0x72, 0x08, 0x69, 0x69, 0xb8, 0x48, 0x01,
        0x00, 0x34, 0x68, 0xfe, 0x81, 0xb9, 0x1f, 0x4d, 0x00, 0x71, 0x42, 0x01, 0x00, 0x54, 0x68,
        0x0a, 0x08, 0x8b, 0xe9, 0x03, 0x00, 0x32, 0x09, 0xb1, 0x01, 0xb9, 0xfd, 0x7b, 0x41, 0xa9,
        0xf4, 0x4f, 0xc2, 0xa8,
    ],
};

/// The member-state setter `(svc, state) -> bool` the ready report calls;
/// the plugin calls it with 7 when the hold ends.
pub const SET_STATE_OFFSET: usize = 0x168db60;
pub const SET_STATE: Site = Site {
    name: "set_state",
    offset: SET_STATE_OFFSET,
    kind: Kind::Called,
    near_only: false,
    guard: [
        0xfc, 0x6f, 0xba, 0xa9, 0xfa, 0x67, 0x01, 0xa9, 0xf8, 0x5f, 0x02, 0xa9, 0xf6, 0x57, 0x03,
        0xa9, 0xf4, 0x4f, 0x04, 0xa9, 0xfd, 0x7b, 0x05, 0xa9, 0xfd, 0x43, 0x01, 0x91, 0xff, 0x83,
        0x08, 0xd1, 0x08, 0xff, 0x93, 0x52, 0x18, 0x00, 0x08, 0x8b, 0x08, 0x38, 0x8d, 0x52, 0x48,
        0x00, 0xa0, 0x72, 0xbc, 0xe1, 0x01, 0xf0, 0x1b, 0x00, 0x08, 0x8b, 0x88, 0x2f, 0x46, 0xf9,
        0x75, 0x03, 0x40, 0xb9,
    ],
};

/// Every site checked before anything is installed.
pub const SITES: [&Site; 5] = [&SEARCH, &READY, &WAIT, &LEAVE, &SET_STATE];

/// The main-image offsets this plugin hooks.
pub const HOOKED_SITES: [usize; 4] = [SEARCH_OFFSET, READY_OFFSET, WAIT_OFFSET, LEAVE_OFFSET];

/// `*(*(*(base + SERVICE) + 8) + 0x88)` is the online service, as `.151b030`
/// loads it for the ready report.
pub const SERVICE: usize = 0x52c4c58;
pub const SERVICE_SLOT: usize = 0x88;
/// Service fields: this console's member ID and the room host's (the wait
/// loop compares their stations, `.1564bf4..1564c5c`), and the round's ack
/// mask the wait loop reads at the hooked site.
pub const SVC_MY_ID: usize = 0x9ff8;
pub const SVC_HOST_ID: usize = 0x9ff0;
pub const SVC_ROUND_MASK: usize = 0x26a34;
/// The service's matchmaking mode: 0 until a search starts a session
/// (`.16b4920` sets it), 2 for Quickplay.
pub const SVC_MODE: usize = 0x74;
/// The online-mode word (`[0x53050f0] & 0x7f000000 == 0x07000000`).
pub const MODE: usize = 0x53050f0;

pub const READY_STATE: u32 = 7;
pub const LOADING_STATE: u32 = 6;

/// The host's full-room wait: the round starts on the 180th full frame
/// (`.1564fd8..1564fe8`: the counter read before its increment is above 178).
pub const FULL_ROOM_WAIT: u32 = 180;

/// Whether the mode word is an online mode.
pub fn online(mode: u32) -> bool {
    mode & 0x7f00_0000 == 0x0700_0000
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
    /// The hold ends: report 7 now.
    Release,
}

/// What happened to the first match's hold, for the diagnostic log.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Note {
    Armed,
    HeldAsJoiner,
    NativeAsHost,
    NativeNoRoom,
    Released,
    DroppedLeave,
    DroppedRound,
    DroppedStale,
}

/// The first-match hold, driven by the four hooks.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Gate {
    /// The next ready report is the first match's of this Quickplay session.
    armed: bool,
    /// Frames held so far, while holding.
    held: Option<u32>,
}

impl Gate {
    pub const fn new() -> Self {
        Self {
            armed: false,
            held: None,
        }
    }

    pub fn armed(&self) -> bool {
        self.armed
    }

    pub fn holding(&self) -> bool {
        self.held.is_some()
    }

    /// The Quickplay sequence starts a search from the character select; the
    /// service's matchmaking mode is `mode` before the call. 0 = a new
    /// Quickplay session: its first match is the one to hold.
    pub fn search(&mut self, mode: u32) -> Option<Note> {
        if mode != 0 {
            return None;
        }
        self.armed = true;
        self.held = None;
        Some(Note::Armed)
    }

    /// The waiting area is about to report `state` with this console in
    /// `role` of an online room (`online`). Returns the state to report and
    /// what happened.
    pub fn ready(&mut self, state: u32, role: Role, online: bool) -> (u32, Option<Note>) {
        if state != READY_STATE {
            return (state, None);
        }
        // A new waiting area: a hold from an earlier one (left some other way) ends.
        let stale = self.held.take().map(|_| Note::DroppedStale);
        if !self.armed || !online {
            return (state, stale);
        }
        self.armed = false;
        match role {
            Role::Joiner => {
                self.held = Some(0);
                (LOADING_STATE, Some(Note::HeldAsJoiner))
            }
            Role::Host => (state, Some(Note::NativeAsHost)),
            Role::NoRoom => (state, Some(Note::NativeNoRoom)),
        }
    }

    /// A frame of the wait loop as the joiner; `round_started` = the ack mask
    /// is set.
    pub fn tick(&mut self, round_started: bool) -> (Tick, Option<Note>) {
        let Some(held) = self.held else {
            return (Tick::Idle, None);
        };
        if round_started {
            self.held = None;
            return (Tick::Idle, Some(Note::DroppedRound));
        }
        let held = held + 1;
        if held >= FULL_ROOM_WAIT {
            self.held = None;
            return (Tick::Release, Some(Note::Released));
        }
        self.held = Some(held);
        (Tick::Holding(held), None)
    }

    /// Leave Battle was confirmed in the waiting area: the hold (and an arm
    /// not used yet) ends.
    pub fn leave(&mut self) -> Option<Note> {
        self.armed = false;
        self.held.take().map(|_| Note::DroppedLeave)
    }
}

/// What `.1564a30` does on a host frame once at least two members are in
/// and every member reports 7.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    /// Starts the battle round.
    Start,
    /// Counts one more full-room frame (and starts after 180).
    Count,
    /// Resets the full-room counter: the room is not full.
    Reset,
}

/// `.1564f4c..1564fe8` as the game runs it on the host: the countdown's
/// result (w20), the online-mode test and the full-room check, with the
/// counter `frames`.
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
/// waiting area starting (its ready report) to the host starting the round,
/// with the host's countdown `out` or still running and `tick` = the frames
/// to the countdown's next report (every 60 frames). `hold` = the joiner
/// holds its 7 for that many frames first. This is the joiner's leave window.
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
    fn crate_is_quickplay_cancel_fix() {
        assert_eq!(env!("CARGO_PKG_NAME"), "quickplay-cancel-fix");
        assert_eq!(env!("CARGO_PKG_VERSION"), "0.1.0");
    }

    fn bl_target(site: &Site, i: usize) -> usize {
        let word = site.stock()[i];
        assert_eq!(word >> 26, 0x25, "{} word {i} is not a bl", site.name);
        let imm = ((word & 0x03ff_ffff) as i32) << 6 >> 4;
        (site.offset as i64 + 4 * i as i64 + imm as i64) as usize
    }

    #[test]
    fn guards_start_with_the_expected_instructions() {
        // orr w1, wzr, #2; mov x0, x8; bl .16b4920
        assert_eq!(SEARCH.stock()[..2], [0x321f_03e1, 0xaa08_03e0]);
        assert_eq!(bl_target(&SEARCH, 2), 0x16b4920);
        // bl .168db60; tbnz w0, #0
        assert_eq!(bl_target(&READY, 0), SET_STATE_OFFSET);
        assert_eq!(READY.stock()[1], 0x3700_0200);
        // ldr x8, [x28, #0xc58]; ...; mov w9, #0x6a34; movk w9, #2, lsl #16 (the ack mask)
        assert_eq!(WAIT.stock()[0], 0xf946_2f88);
        assert_eq!(WAIT.stock()[3], 0x528d_4689);
        assert_eq!(WAIT.stock()[4], 0x72a0_0049);
        assert_eq!(0x2_0000 | ((WAIT.stock()[3] >> 5) & 0xffff) as usize, SVC_ROUND_MASK);
        // adrp x8, 0x52c4000; ldr x8, [x8, #0xc58]; ...; the same mask read, then cbz
        assert_eq!(LEAVE.stock()[1], 0xf946_2d08);
        assert_eq!(LEAVE.stock()[4..6], WAIT.stock()[3..5]);
        // stp x28, x27, [sp, #-0x60]!
        assert_eq!(SET_STATE.stock()[0], 0xa9ba_6ffc);
    }

    /// Where each site would be with the image at a page-aligned base.
    fn at(site: &Site) -> u64 {
        0x80_0000_0000 + site.offset as u64
    }

    #[test]
    fn sites_are_checked_by_kind() {
        let hooked: Vec<_> = SITES.iter().filter(|s| s.kind == Kind::Hooked).map(|s| s.offset).collect();
        assert_eq!(hooked, HOOKED_SITES);
        assert_eq!(SET_STATE.kind, Kind::Called);
        for site in SITES {
            assert!(site.usable(&site.stock(), at(site)), "{}", site.name);
        }
        // TrainingRooms fingerprints .236b470+0x104 and .1851480+0x100: no site in them.
        for (start, len) in [(0x236b470, 0x104), (0x1851480, 0x100), (0x18512c0, 0x50), (0x1851310, 0x30)] {
            for site in SITES {
                assert!(site.offset + 64 <= start || site.offset >= start + len, "{}", site.name);
            }
        }
    }

    #[test]
    fn other_plugins_hooks_are_accepted_and_changes_are_not() {
        for site in SITES {
            let mut words = site.stock();
            words[0] = 0x1400_0100; // another plugin's one-word branch
            assert!(site.usable(&words, at(site)), "{}", site.name);
            let mut words = site.stock();
            words[3] ^= 0x10;
            assert!(!site.usable(&words, at(site)), "{}", site.name);
        }
        // Another plugin's far jump over the ready report would cover .151b04c.
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
        assert!(online(0x0701_0800));
        assert!(!online(0x0601_0800));
    }

    /// The character select's Ready starts a new Quickplay session.
    fn new_session(gate: &mut Gate) {
        assert_eq!(gate.search(0), Some(Note::Armed));
    }

    #[test]
    fn arms_once_per_quickplay_entry() {
        let mut gate = Gate::new();
        assert!(!gate.armed());
        // A search in a session already under way (mode 2): no arm.
        assert_eq!(gate.search(2), None);
        assert!(!gate.armed());
        new_session(&mut gate);
        assert!(gate.armed());
        // The first ready report uses it, in either role.
        assert_eq!(gate.ready(READY_STATE, Role::Host, true), (READY_STATE, Some(Note::NativeAsHost)));
        assert!(!gate.armed());
        // Later searches of the session: no new arm, nothing held.
        assert_eq!(gate.search(2), None);
        assert_eq!(gate.ready(READY_STATE, Role::Joiner, true), (READY_STATE, None));
        // A new session from the character select arms again.
        new_session(&mut gate);
        assert!(gate.armed());
    }

    #[test]
    fn holds_only_as_the_joiner() {
        for (role, reported, note, holding) in [
            (Role::Joiner, LOADING_STATE, Note::HeldAsJoiner, true),
            (Role::Host, READY_STATE, Note::NativeAsHost, false),
            (Role::NoRoom, READY_STATE, Note::NativeNoRoom, false),
        ] {
            let mut gate = Gate::new();
            new_session(&mut gate);
            assert_eq!(gate.ready(READY_STATE, role, true), (reported, Some(note)), "{role:?}");
            assert_eq!(gate.holding(), holding, "{role:?}");
            assert!(!gate.armed(), "{role:?}");
        }
        // Offline, or a state other than 7: untouched and still armed.
        let mut gate = Gate::new();
        new_session(&mut gate);
        assert_eq!(gate.ready(READY_STATE, Role::Joiner, false), (READY_STATE, None));
        assert_eq!(gate.ready(LOADING_STATE, Role::Joiner, true), (LOADING_STATE, None));
        assert!(gate.armed());
    }

    #[test]
    fn releases_after_the_full_room_wait() {
        let mut gate = Gate::new();
        new_session(&mut gate);
        gate.ready(READY_STATE, Role::Joiner, true);
        for frame in 1..FULL_ROOM_WAIT {
            assert_eq!(gate.tick(false), (Tick::Holding(frame), None));
        }
        assert_eq!(gate.tick(false), (Tick::Release, Some(Note::Released)));
        assert_eq!(gate.tick(false), (Tick::Idle, None));
        assert!(!gate.holding());
    }

    #[test]
    fn a_leave_drops_the_hold() {
        let mut gate = Gate::new();
        new_session(&mut gate);
        gate.ready(READY_STATE, Role::Joiner, true);
        gate.tick(false);
        assert_eq!(gate.leave(), Some(Note::DroppedLeave));
        assert_eq!(gate.tick(false), (Tick::Idle, None));
        // A round started anyway.
        let mut gate = Gate::new();
        new_session(&mut gate);
        gate.ready(READY_STATE, Role::Joiner, true);
        assert_eq!(gate.tick(true), (Tick::Idle, Some(Note::DroppedRound)));
        // The waiting area ended some other way: the next ready report ends the stale hold.
        let mut gate = Gate::new();
        new_session(&mut gate);
        gate.ready(READY_STATE, Role::Joiner, true);
        assert_eq!(gate.ready(READY_STATE, Role::Joiner, true), (READY_STATE, Some(Note::DroppedStale)));
        assert!(!gate.holding());
        // A leave before the first ready report drops the arm too.
        let mut gate = Gate::new();
        new_session(&mut gate);
        assert_eq!(gate.leave(), None);
        assert!(!gate.armed());
    }

    #[test]
    fn later_matches_are_native() {
        let mut gate = Gate::new();
        new_session(&mut gate);
        gate.ready(READY_STATE, Role::Joiner, true);
        while gate.tick(false).0 != Tick::Release {}
        // Every later ready report in the session, whatever the role.
        for role in [Role::Joiner, Role::Host, Role::Joiner] {
            assert_eq!(gate.search(2), None);
            assert_eq!(gate.ready(READY_STATE, role, true), (READY_STATE, None));
            assert_eq!(gate.tick(false), (Tick::Idle, None));
        }
    }

    #[test]
    fn host_step_follows_the_game() {
        assert_eq!(host_step(true, true, true, 0), Step::Start);
        assert_eq!(host_step(true, false, true, 0), Step::Count);
        assert_eq!(host_step(false, true, true, 178), Step::Count);
        assert_eq!(host_step(false, true, true, 179), Step::Start);
        assert_eq!(host_step(false, true, false, 179), Step::Reset);
        // `cmp w8, #178` at .1564fe0 (the guard's neighbour .15669f8 has the same).
        assert_eq!(FULL_ROOM_WAIT - 2, 178);
    }

    /// The parity table: the joiner's leave window against a vanilla host, on
    /// every countdown phase. An early arrival (countdown running) gets the
    /// full-room wait natively; a late one (countdown out) gets at most a
    /// second natively and at least the full-room wait with the hold.
    #[test]
    fn the_first_match_waits_like_an_early_arrival() {
        for tick in 0..60 {
            let early = leave_window(false, tick, 0);
            let late_native = leave_window(true, tick, 0);
            let late_held = leave_window(true, tick, FULL_ROOM_WAIT);
            assert_eq!(early, FULL_ROOM_WAIT - 1, "tick {tick}");
            assert!(late_native < 60, "tick {tick}");
            assert!(late_held >= early, "tick {tick}: {late_held} < {early}");
            assert!(late_held < FULL_ROOM_WAIT + 60, "tick {tick}");
        }
        // The cost when the host's countdown is still running: its own wait follows the hold.
        assert_eq!(leave_window(false, 0, FULL_ROOM_WAIT), 2 * FULL_ROOM_WAIT - 1);
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
