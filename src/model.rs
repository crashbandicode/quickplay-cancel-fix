//! Host-testable half: the hook site, the code the plugin relies on, and the
//! rematch count it gives a lobby entered from the character select.
//!
//! In Quickplay the room keeps, per member slot, a count of the battles this
//! console has played with it (`0x5313fb4 + slot * 0xb0`, the slot's station
//! at `0x5313f08 + slot * 0xb0`). Entering the lobby from the character select
//! resets every slot to 1 (IngameCallSequenceScene state 0, `.230d000`); each
//! battle bumps the present slots (`.2314b00`, `.2315f40`); a member that
//! leaves gets its slot reset to 1 (`.16af2e0`). Answering "Rematch?" Yes goes
//! back to the lobby without that reset (state 5 to 3), so this console's own
//! slot is 2 or more there, and a new opponent's loading can then be left the
//! game's own way: Pause, Leave Battle, Yes. After a character-select entry
//! the own slot is 1 and that loading cannot be left.
//!
//! The plugin gives the own slot of a character-select lobby the count a
//! rematch lobby has: 1 becomes 2, on the first waiting-area frame where this
//! console's slot is known. Nothing else is written; a slot already at 2 or
//! more (every rematch lobby) is left as it is, and so is every other slot.
//!
//! Candidate R (research, not released): the raised count lasts only while
//! the waiting area is up. On the frame PracticeScene requests its exit (the
//! match starts, or the player left), the plugin puts the native 1 back, so
//! the post-battle bump makes it 2, as the opponent counts this console's
//! slot. v0.2.0 left it raised, so after the first match this console
//! counted its own slot 3 while the opponent counted it 2.

use crate::{chain, hold};

pub const SUPPORTED_VERSION: &str = "13.0.5";

/// How a site is checked before anything is installed. Other plugins' hooks
/// are accepted as `chain` reads them; any other change is not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// One of the plugin's inline hooks goes here: another plugin's hook on it
    /// is chained onto (`chain::entry`).
    Hooked,
    /// A hooked site where code jumps into the words a far jump would
    /// replace: only a one-word branch may go there, so another plugin's far
    /// jump there leaves it alone too.
    HookedNearOnly,
    /// Code the plugin relies on or calls (it reads or writes the data this
    /// code reads or writes): other plugins' hooks in it are fine.
    Relied,
}

impl Kind {
    pub fn hooked(self) -> bool {
        matches!(self, Self::Hooked | Self::HookedNearOnly)
    }
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
            Kind::HookedNearOnly => !matches!(
                chain::entry(&stock, words, address),
                chain::Entry::Foreign | chain::Entry::Chain(_)
            ),
            Kind::Relied => chain::usable(&stock, words, address, 0),
        }
    }
}

/// PracticeScene's running state (`.1851660`, every waiting-area frame), after
/// its update call: `ldrb w8, [x19, #0xf0]` (its exit request), x19 = the
/// PracticeScene context. Nothing jumps into the 20 bytes from here; no flags
/// are live.
pub const WAIT_OFFSET: usize = 0x1851674;
/// The register holding the PracticeScene context at `WAIT`, and its exit
/// request byte there (non-zero once the scene is to end).
pub const WAIT_CONTEXT_REGISTER: usize = 19;
pub const PRACTICE_EXIT: usize = 0xf0;
pub const WAIT: Site = Site {
    name: "wait",
    offset: WAIT_OFFSET,
    kind: Kind::Hooked,
    guard: [
        0x68, 0xc2, 0x43, 0x39, 0x28, 0x01, 0x00, 0x34, 0x68, 0xfe, 0x81, 0xb9, 0x1f, 0x4d, 0x00,
        0x71, 0x22, 0x01, 0x00, 0x54, 0x68, 0x0a, 0x08, 0x8b, 0xe9, 0x03, 0x1f, 0x32, 0x09, 0xb1,
        0x01, 0xb9, 0x68, 0xf6, 0x40, 0xb9, 0x68, 0x12, 0x02, 0xb9, 0xfd, 0x7b, 0x41, 0xa9, 0xf3,
        0x07, 0x42, 0xf8, 0xc0, 0x03, 0x5f, 0xd6, 0xea, 0xc3, 0x85, 0x94, 0x00, 0x00, 0x00, 0x00,
        0xe8, 0xd6, 0x01, 0xd0,
    ],
};

/// The waiting battle's member pass (`.15637a0`), its count load for a slot
/// (`.1563ef8`): `mov w9, #0xee34; movk w9, #4, lsl #16; ldr w8, [x8, x9];
/// cmp w8, #1; b.hi`. On entry x8 is the slot's record (`madd x8, x26, #0xb0,
/// x23`, x23 = `COUNT_OBJECT`), x26 the slot. With a count below 2 the pass
/// names that member "???" (`mel_mockbattle_name_unknown`); on Eden it runs
/// for the opponents' slots only (qcf-xfz5: never for the own slot). 8-byte
/// aligned, so a far jump takes these four words: no PC-relative
/// instruction, no branch into them (`re/branch_into.py`), the flags are set
/// by their own `cmp`, and `.15637a0` does not use x17.
pub const GATE_OFFSET: usize = 0x1563ef8;
pub const GATE: Site = Site {
    name: "names",
    offset: GATE_OFFSET,
    kind: Kind::Hooked,
    guard: [
        0x89, 0xc6, 0x9d, 0x52, 0x89, 0x00, 0xa0, 0x72, 0x08, 0x69, 0x69, 0xb8, 0x1f, 0x05, 0x00,
        0x71, 0x08, 0x04, 0x00, 0x54, 0x08, 0xed, 0x01, 0xd0, 0x08, 0xf1, 0x40, 0xb9, 0xe9, 0xdf,
        0x9f, 0x52, 0xc9, 0x1f, 0xbf, 0x72, 0x09, 0x01, 0x09, 0x0b, 0x3f, 0x09, 0x00, 0x71, 0x23,
        0x01, 0x00, 0x54, 0x09, 0x00, 0x81, 0x52, 0x29, 0xe0, 0xa0, 0x72, 0x1f, 0x01, 0x09, 0x6b,
        0xa0, 0x00, 0x00, 0x54,
    ],
};
/// At `GATE`: the register with the slot's record, the one with the slot, and
/// the record-relative offset the load adds (the count).
pub const GATE_RECORD_REGISTER: usize = 8;
pub const GATE_SLOT_REGISTER: usize = 26;
pub const GATE_COUNT_OFFSET: usize = 0x4ee34;
/// The object the records are in: object + `GATE_COUNT_OFFSET` is slot 0's
/// count.
pub const COUNT_OBJECT: usize = 0x52c5180;

/// The character-select entry's reset (IngameCallSequenceScene state 0,
/// `.230c160`): `adrp x9, 0x5313000; add x9, x9, #0xfb4; orr w10, wzr, #1`,
/// then `str w10` at `+0`, `+0xb0`, ... `+0x4d0`: every slot's count to 1.
/// Fixes the count table's address and stride.
pub const COUNT_RESET: Site = Site {
    name: "count_reset",
    offset: 0x230d000,
    kind: Kind::Relied,
    guard: [
        0x29, 0x80, 0x01, 0xd0, 0x29, 0xd1, 0x3e, 0x91, 0xea, 0x03, 0x00, 0x32, 0x2a, 0x01, 0x00,
        0xb9, 0x2a, 0xb1, 0x00, 0xb9, 0x2a, 0x61, 0x01, 0xb9, 0x2a, 0x11, 0x02, 0xb9, 0x2a, 0xc1,
        0x02, 0xb9, 0x2a, 0x71, 0x03, 0xb9, 0x2a, 0x21, 0x04, 0xb9, 0x2a, 0xd1, 0x04, 0xb9, 0xa8,
        0x00, 0x00, 0xb4, 0x00, 0x21, 0x16, 0x91, 0x08, 0xc5, 0x42, 0xf9, 0x08, 0x35, 0x40, 0xf9,
        0x00, 0x01, 0x3f, 0xd6,
    ],
};

/// The bump after a battle (`.2314b00`): `ldrb w10, [0x5313f08]` (slot 0's
/// station), its bit in the service's member mask, then `ldr w11, [0x5313fb4];
/// add w11, w11, #1; str w11`. Fixes the station byte's place in the slot.
pub const COUNT_BUMP: Site = Site {
    name: "count_bump",
    offset: 0x2315f40,
    kind: Kind::Relied,
    guard: [
        0xea, 0x7f, 0x01, 0xd0, 0x4a, 0x21, 0x7c, 0x39, 0x09, 0x05, 0x40, 0xf9, 0x5f, 0xfd, 0x03,
        0x71, 0x80, 0x01, 0x00, 0x54, 0x2b, 0x45, 0x40, 0xf9, 0x0c, 0x00, 0x94, 0x52, 0x6b, 0x69,
        0x6c, 0xb8, 0xec, 0x03, 0x00, 0x32, 0x8a, 0x21, 0xca, 0x1a, 0x7f, 0x01, 0x0a, 0x6a, 0xa0,
        0x00, 0x00, 0x54, 0xea, 0x7f, 0x01, 0xd0, 0x4b, 0xb5, 0x4f, 0xb9, 0x6b, 0x05, 0x00, 0x11,
        0x4b, 0xb5, 0x0f, 0xb9,
    ],
};

/// The station of a member ID (`.16f6340(svc + 0xa010, id)`): the index of
/// the 16 station records (`+8`, stride 0x78) holding `id`. Fixes that layout,
/// which the plugin reads for its own ID.
pub const STATION_OF: Site = Site {
    name: "station_of",
    offset: 0x16f6340,
    kind: Kind::Relied,
    guard: [
        0x61, 0x02, 0x00, 0xb4, 0xe8, 0x03, 0x00, 0xaa, 0x09, 0x8d, 0x40, 0xf8, 0x3f, 0x01, 0x01,
        0xeb, 0xe9, 0x03, 0x08, 0xaa, 0x60, 0x09, 0x00, 0x54, 0xe9, 0x03, 0x00, 0xaa, 0x2a, 0x0d,
        0x48, 0xf8, 0x5f, 0x01, 0x01, 0xeb, 0xe0, 0x08, 0x00, 0x54, 0xe9, 0x03, 0x00, 0xaa, 0x2a,
        0x8d, 0x4f, 0xf8, 0x5f, 0x01, 0x01, 0xeb, 0x60, 0x08, 0x00, 0x54, 0x09, 0xb8, 0x40, 0xf9,
        0x3f, 0x01, 0x01, 0xeb,
    ],
};

/// The waiting battle's member pass (`.15637a0`) reading the online-mode word
/// (`adrp x8, 0x5305000; ldr w8, [x8, #0xf0]`) for its Quickplay modes.
/// Fixes the mode word's address.
pub const MODE_READ: Site = Site {
    name: "mode_read",
    offset: 0x1563f0c,
    kind: Kind::Relied,
    guard: [
        0x08, 0xed, 0x01, 0xd0, 0x08, 0xf1, 0x40, 0xb9, 0xe9, 0xdf, 0x9f, 0x52, 0xc9, 0x1f, 0xbf,
        0x72, 0x09, 0x01, 0x09, 0x0b, 0x3f, 0x09, 0x00, 0x71, 0x23, 0x01, 0x00, 0x54, 0x09, 0x00,
        0x81, 0x52, 0x29, 0xe0, 0xa0, 0x72, 0x1f, 0x01, 0x09, 0x6b, 0xa0, 0x00, 0x00, 0x54, 0x09,
        0x80, 0x80, 0x52, 0x29, 0xe0, 0xa0, 0x72, 0x1f, 0x01, 0x09, 0x6b, 0x21, 0x02, 0x00, 0x54,
        0xe8, 0x0f, 0x40, 0xf9,
    ],
};

/// The member pass's own-slot test (`.1563ed4`): `adrp x0, 0x5308000; add x0,
/// x0, #0x760; mov w1, w26; bl .179ed00` on the lobby's member table, before
/// the count load at `GATE`. Fixes the table's address. The table knows a
/// joiner's slot from the start of its waiting area, while the slot table at
/// `SLOT_STATION` gets the joiner's station only after its ready report
/// (qcf-shd1: 0x00 for the joiner's slot during the hold).
pub const MEMBER_TEST: Site = Site {
    name: "member_test",
    offset: 0x1563ed4,
    kind: Kind::Relied,
    guard: [
        0x20, 0xed, 0x01, 0xb0, 0x00, 0x80, 0x1d, 0x91, 0xe1, 0x03, 0x1a, 0x2a, 0x88, 0xeb, 0x08,
        0x94, 0x20, 0x05, 0x00, 0x37, 0x5f, 0x1f, 0x00, 0xf1, 0x08, 0x01, 0x00, 0x54, 0x08, 0x16,
        0x80, 0x52, 0x48, 0x5f, 0x08, 0x9b, 0x89, 0xc6, 0x9d, 0x52, 0x89, 0x00, 0xa0, 0x72, 0x08,
        0x69, 0x69, 0xb8, 0x1f, 0x05, 0x00, 0x71, 0x08, 0x04, 0x00, 0x54, 0x08, 0xed, 0x01, 0xd0,
        0x08, 0xf1, 0x40, 0xb9,
    ],
};
/// `.179ed00(table, slot)` on a slot below 8: the slot's entry, 0x1c8 bytes
/// per slot, whose `+0x10` it passes on (`mov w9, #0x1c8; madd; add x0, x8,
/// #16`). Fixes the entry's size and the station byte's place in it.
pub const MEMBER_ENTRY: Site = Site {
    name: "member_entry",
    offset: 0x179ed88,
    kind: Kind::Relied,
    guard: [
        0x68, 0x1e, 0x00, 0x12, 0x1f, 0x21, 0x00, 0x71, 0x22, 0x09, 0x00, 0x54, 0x68, 0x1e, 0x40,
        0x92, 0x09, 0x39, 0x80, 0x52, 0x08, 0x55, 0x09, 0x9b, 0xfd, 0x7b, 0x43, 0xa9, 0x00, 0x41,
        0x00, 0x91, 0xf4, 0x4f, 0x42, 0xa9, 0xf6, 0x57, 0x41, 0xa9, 0xf7, 0x07, 0x44, 0xf8, 0x5f,
        0xff, 0xff, 0x17, 0xe8, 0x03, 0x33, 0x2a, 0x1f, 0x1d, 0x00, 0x72, 0xa0, 0x06, 0x00, 0x54,
        0xf5, 0x46, 0x40, 0xf9,
    ],
};
/// The lobby's member table: per slot an entry of `MEMBER_STRIDE` bytes with
/// the member's station at `MEMBER_STATION` (`NO_STATION`: nobody).
pub const MEMBER_TABLE: usize = 0x5308760;
pub const MEMBER_STRIDE: usize = 0x1c8;
pub const MEMBER_STATION: usize = 0x10;

/// Every site checked before anything is installed.
pub const SITES: [&Site; 13] = [
    &WAIT,
    &GATE,
    &COUNT_RESET,
    &COUNT_BUMP,
    &STATION_OF,
    &MODE_READ,
    &MEMBER_TEST,
    &MEMBER_ENTRY,
    &hold::READY,
    &hold::LEAVE,
    &hold::SET_STATE,
    &hold::INPUT_FREE,
    &hold::MENU_GATE,
];

/// The main-image offsets this plugin hooks.
pub const HOOKED_SITES: [usize; 4] = [WAIT_OFFSET, GATE_OFFSET, hold::READY_OFFSET, hold::LEAVE_OFFSET];

/// `*(*(*(base + SERVICE) + 8) + 0x88)` is the online service.
pub const SERVICE: usize = 0x52c4c58;
pub const SERVICE_SLOT: usize = 0x88;
/// This console's member ID in the service, and the station records
/// `.16f6340` searches (`svc + 0xa010`: IDs at `+8 + station * 0x78`).
pub const SVC_MY_ID: usize = 0x9ff8;
pub const SVC_STATIONS: usize = 0xa018;
pub const STATION_STRIDE: usize = 0x78;
pub const STATIONS: usize = 16;
pub const NO_STATION: u8 = 0xff;

/// The online-mode word, and Quickplay's value in it.
pub const MODE: usize = 0x5305000 + 0xf0;
pub const QUICKPLAY: u32 = 0x0701_0101;

/// The per-slot battle counts and stations.
pub const SLOTS: usize = 8;
pub const SLOT_STRIDE: usize = 0xb0;
pub const SLOT_COUNT: usize = 0x5313000 + 0xfb4;
pub const SLOT_STATION: usize = 0x5313000 + 0xf08;
/// What the character-select entry resets a slot to, and what one battle
/// (the rematch path) leaves on it.
pub const FRESH_COUNT: u32 = 1;
pub const REMATCH_COUNT: u32 = 2;

/// The station whose record holds `id` (`.16f6340`), or `NO_STATION`.
pub fn station_of(records: &[u64; STATIONS], id: u64) -> u8 {
    if id == 0 {
        return NO_STATION;
    }
    records.iter().position(|r| *r == id).map_or(NO_STATION, |s| s as u8)
}

/// The slot this console's station occupies.
pub fn own_slot(stations: &[u8; SLOTS], mine: u8) -> Option<usize> {
    if mine == NO_STATION {
        return None;
    }
    stations.iter().position(|s| *s == mine)
}

/// This console's slot in the lobby: from the member table (`MEMBER_TABLE`),
/// which has it from the start of the waiting area, else from the slot
/// table (`SLOT_STATION`, set for a joiner only after its ready report).
pub fn own_slot_in(members: &[u8; SLOTS], slots: &[u8; SLOTS], mine: u8) -> Option<usize> {
    own_slot(members, mine).or_else(|| own_slot(slots, mine))
}

/// The count to write into this console's slot of a Quickplay lobby, if any:
/// a character-select entry's fresh 1 becomes the rematch path's 2; anything
/// else (a rematch lobby's 2 or more, an unset 0) stays.
pub fn adopt(mode: u32, count: u32) -> Option<u32> {
    (mode == QUICKPLAY && count == FRESH_COUNT).then_some(REMATCH_COUNT)
}

/// The count to put back into a slot this plugin raised, when the waiting
/// area ends: the native 1, if the slot still holds the 2 the plugin wrote
/// (anything else is the game's own value and stays).
pub fn restore(count: u32) -> Option<u32> {
    (count == REMATCH_COUNT).then_some(FRESH_COUNT)
}

/// What the member pass's load at `GATE` should read for `slot` instead of
/// its stored `count`: in Quickplay, for an opponent's slot (this console's
/// own slot known and not `slot`) whose stored count is below 2 (the "???"
/// rule), the rematch count, so the opponent's name shows; `None` keeps the
/// stored value. The stored counts are not written.
pub fn shown(mode: u32, slot: usize, own: Option<usize>, count: u32) -> Option<u32> {
    let opponent = own.is_some_and(|own| own != slot);
    (mode == QUICKPLAY && opponent && count < REMATCH_COUNT).then_some(REMATCH_COUNT)
}

/// One waiting-area frame: the slot the plugin raised and has not put back
/// (`raised`), the online mode, this console's slot, the slots' counts and
/// whether PracticeScene requested its exit. Returns what the plugin keeps
/// raised after this frame and the count it writes, if any.
pub fn frame(
    raised: Option<usize>,
    mode: u32,
    own: Option<usize>,
    counts: &[u32; SLOTS],
    exiting: bool,
) -> (Option<usize>, Option<(usize, u32)>) {
    if exiting {
        let write = raised.and_then(|slot| restore(counts[slot]).map(|count| (slot, count)));
        return (None, write);
    }
    if let Some(slot) = raised {
        if own.is_some_and(|own| own != slot) {
            // This console is in another room now (its search joined another host's room after waiting in its own,
            // qcf-xfz7 m1): the raised slot belongs to another member there. Put the native 1 back; the next frame
            // raises the new own slot.
            return (None, restore(counts[slot]).map(|count| (slot, count)));
        }
        // Still raised: nothing to do, unless the game wrote the slot meanwhile.
        return (Some(slot).filter(|s| counts[*s] == REMATCH_COUNT), None);
    }
    match own.and_then(|slot| adopt(mode, counts[slot]).map(|count| (slot, count))) {
        Some((slot, count)) => (Some(slot), Some((slot, count))),
        None => (None, None),
    }
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
    }

    /// The page an `adrp` at `site.offset + 4 * i` loads.
    fn adrp_page(site: &Site, i: usize) -> usize {
        let word = site.stock()[i];
        assert_eq!(word & 0x9f00_0000, 0x9000_0000, "{} word {i} is not an adrp", site.name);
        let imm = (((word >> 5) & 0x7ffff) << 2 | (word >> 29) & 3) as i64;
        let imm = if imm & (1 << 20) != 0 { imm - (1 << 21) } else { imm };
        (((site.offset + 4 * i) as i64 & !0xfff) + (imm << 12)) as usize
    }

    /// The byte offset of an unsigned-immediate load or store.
    fn ldst_offset(word: u32) -> usize {
        (((word >> 10) & 0xfff) << (word >> 30)) as usize
    }

    #[test]
    fn the_relied_on_code_fixes_the_addresses() {
        // ldrb w8, [x19, #0xf0]; cbz w8
        assert_eq!(WAIT.stock()[0], 0x3943_c268);
        // count_reset: adrp x9; add x9, x9, #0xfb4; orr w10, wzr, #1; str w10, [x9, #slot * 0xb0] for 8 slots
        let reset = COUNT_RESET.stock();
        assert_eq!(adrp_page(&COUNT_RESET, 0) + ((reset[1] >> 10) & 0xfff) as usize, SLOT_COUNT);
        assert_eq!(reset[2], 0x3200_03ea);
        for slot in 0..SLOTS {
            assert_eq!(reset[3 + slot] & 0xffc0_03ff, 0xb900_012a, "slot {slot}");
            assert_eq!(ldst_offset(reset[3 + slot]), slot * SLOT_STRIDE, "slot {slot}");
        }
        assert_eq!(FRESH_COUNT, 1);
        // count_bump: adrp x10; ldrb w10, [x10, #0xf08] ... adrp x10; ldr w11, [x10, #0xfb4]; add w11, w11, #1; str w11
        let bump = COUNT_BUMP.stock();
        assert_eq!(adrp_page(&COUNT_BUMP, 0) + ldst_offset(bump[1]), SLOT_STATION);
        assert_eq!(bump[1] >> 22, 0x0e5, "ldrb");
        assert_eq!(adrp_page(&COUNT_BUMP, 12) + ldst_offset(bump[13]), SLOT_COUNT);
        assert_eq!(bump[14], 0x1100_056b);
        assert_eq!(bump[15], 0xb90f_b54b);
        assert_eq!(SLOT_COUNT - SLOT_STATION, 0xac);
        // station_of: cbz x1; mov x8, x0; ldr x9, [x8, #8]!; ... ldr x10, [x9, #0x80]!; ... ldr x10, [x9, #0xf8]!
        let station = STATION_OF.stock();
        assert_eq!(station[2], 0xf840_8d09);
        assert_eq!(station[7], 0xf848_0d2a);
        assert_eq!(station[11], 0xf84f_8d2a);
        assert_eq!(SVC_STATIONS - 0xa010, 8);
        assert_eq!(0x80 - 8, STATION_STRIDE);
        // mode_read: adrp x8, 0x5305000; ldr w8, [x8, #0xf0]; the Quickplay modes it compares with
        assert_eq!(adrp_page(&MODE_READ, 0) + ldst_offset(MODE_READ.stock()[1]), MODE);
        let quickplay_low = 0u32.wrapping_sub(0xf8fe_feff);
        assert_eq!(quickplay_low, QUICKPLAY);
        // member_test: adrp x0, 0x5308000; add x0, x0, #0x760; mov w1, w26; bl .179ed00; then the count load (GATE)
        let test = MEMBER_TEST.stock();
        assert_eq!(adrp_page(&MEMBER_TEST, 0) + ((test[1] >> 10) & 0xfff) as usize, MEMBER_TABLE);
        assert_eq!(test[2], 0x2a1a_03e1, "mov w1, w26: the slot");
        let imm = ((test[3] & 0x03ff_ffff) as i32) << 6 >> 4;
        assert_eq!((MEMBER_TEST.offset as i64 + 12 + imm as i64) as usize, 0x179ed00);
        assert_eq!(GATE_OFFSET - MEMBER_TEST.offset, 9 * 4);
        assert_eq!(MEMBER_TEST.guard[36..64], GATE.guard[..28]);
        // member_entry: cmp w8, #8; b.hs; mov w9, #0x1c8; madd x8, x8, x9, x21; ...; add x0, x8, #16
        let entry = MEMBER_ENTRY.stock();
        assert_eq!(entry[1], 0x7100_211f, "cmp w8, #8");
        assert_eq!(SLOTS, 8);
        assert_eq!(((entry[4] >> 5) & 0xffff) as usize, MEMBER_STRIDE);
        assert_eq!(entry[5], 0x9b09_5508);
        assert_eq!(((entry[7] >> 10) & 0xfff) as usize, MEMBER_STATION);
    }

    /// qcf-shd1 h3l, the joiner held at 6: the member table had both stations (slot 0 the host's 0, slot 1 this
    /// console's 1), the slot table had 0 for both, so 0.3.0's first build raised nothing and the tab didn't show.
    #[test]
    fn the_own_slot_comes_from_the_member_table_first() {
        let mut members = [NO_STATION; SLOTS];
        let slots = [0u8; SLOTS];
        assert_eq!(own_slot_in(&members, &slots, 1), None, "the joiner, before anything is known");
        assert_eq!(own_slot_in(&members, &slots, 0), Some(0), "no members yet: the slot table, as 0.2.1");
        members[0] = 0;
        members[1] = 1;
        assert_eq!(own_slot_in(&members, &slots, 1), Some(1), "the held joiner");
        assert_eq!(own_slot_in(&members, &slots, 0), Some(0), "the host");
        // After the joiner's ready report both tables agree.
        let mut slots = [NO_STATION; SLOTS];
        slots[0] = 0;
        slots[1] = 1;
        assert_eq!(own_slot_in(&members, &slots, 1), Some(1));
        assert_eq!(own_slot_in(&members, &slots, NO_STATION), None);
        // qcf-xfz7 m1 (a room switch): the member table follows the room; the joiner is slot 1 there.
        let members = [0, 1, NO_STATION, NO_STATION, NO_STATION, NO_STATION, NO_STATION, NO_STATION];
        let stale = [1, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(own_slot_in(&members, &stale, 1), Some(1));
    }

    /// Where each site would be with the image at a page-aligned base.
    fn at(site: &Site) -> u64 {
        0x80_0000_0000 + site.offset as u64
    }

    #[test]
    fn sites_are_checked_by_kind() {
        let hooked: Vec<_> = SITES.iter().filter(|s| s.kind.hooked()).map(|s| s.offset).collect();
        assert_eq!(hooked, HOOKED_SITES);
        for site in SITES {
            assert!(site.usable(&site.stock(), at(site)), "{}", site.name);
        }
        // TrainingRooms fingerprints .236b470+0x104, .1851480+0x100, .18512c0+0x50 and .1851310+0x30: the hook
        // is outside them.
        for (start, len) in [(0x236b470, 0x104), (0x1851480, 0x100), (0x18512c0, 0x50), (0x1851310, 0x30)] {
            for site in SITES.iter().filter(|s| s.kind.hooked()) {
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
    }

    #[test]
    fn the_station_is_the_record_holding_the_id() {
        let mut records = [0u64; STATIONS];
        records[0] = 0xc100_710d_6166_8319;
        records[3] = 0xc100_aaaa_bbbb_cccc;
        assert_eq!(station_of(&records, 0xc100_710d_6166_8319), 0);
        assert_eq!(station_of(&records, 0xc100_aaaa_bbbb_cccc), 3);
        assert_eq!(station_of(&records, 0x1234), NO_STATION);
        assert_eq!(station_of(&records, 0), NO_STATION, "no room: no own ID");
    }

    #[test]
    fn the_own_slot_holds_this_consoles_station() {
        let stations = [0, 1, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        assert_eq!(own_slot(&stations, 0), Some(0));
        assert_eq!(own_slot(&stations, 1), Some(1));
        assert_eq!(own_slot(&stations, 2), None);
        assert_eq!(own_slot(&stations, NO_STATION), None);
    }

    /// The counts seen on Eden (qcf-xrm06): a character-select lobby has the
    /// own slot at 1; the rematch lobbies at 2; a joiner's slot is 1 in both.
    #[test]
    fn a_character_select_lobby_takes_the_rematch_count() {
        assert_eq!(adopt(QUICKPLAY, 1), Some(REMATCH_COUNT));
        // A rematch lobby (one or more battles) and an unset slot stay as they are.
        for count in [0, 2, 3, 7] {
            assert_eq!(adopt(QUICKPLAY, count), None, "count {count}");
        }
        // Other online modes and offline: untouched.
        for mode in [0, 0x0701_0102, 0x0701_0400, 0x0701_0800, 0x0202_0000] {
            assert_eq!(adopt(mode, 1), None, "mode {mode:#x}");
        }
        // Writing it again on the next frame changes nothing.
        assert_eq!(adopt(QUICKPLAY, REMATCH_COUNT), None);
    }

    /// The room's per-slot counts as the game keeps them: the character
    /// select's reset (`.230d000`), the bump of every present slot after a
    /// battle (`.2315f40`), a leaving member's slot back to 1 (`.16af2e0`);
    /// a rematch "Yes" goes back to the lobby without a reset.
    struct Room {
        counts: [u32; SLOTS],
    }

    impl Room {
        fn from_character_select() -> Self {
            Self { counts: [FRESH_COUNT; SLOTS] }
        }

        fn battle(&mut self, present: &[usize]) {
            for slot in present {
                self.counts[*slot] += 1;
            }
        }

        fn member_left(&mut self, slot: usize) {
            self.counts[slot] = FRESH_COUNT;
        }

        /// The plugin on a waiting-area frame, this console in `own`.
        fn with_plugin(mut self, own: usize) -> Self {
            if let Some(count) = adopt(QUICKPLAY, self.counts[own]) {
                self.counts[own] = count;
            }
            self
        }
    }

    /// Whether a new opponent's loading can be left: on Eden (qcf-xrm06 m0,
    /// m5, m3; qcf-xrm07 after, before) it followed this console's own count,
    /// shown at 2 or more and hidden at 1, whatever the joiner's slot held.
    fn leave_offered(room: &Room, own: usize) -> bool {
        room.counts[own] >= REMATCH_COUNT
    }

    /// Parity: a lobby entered from the character select, with the plugin,
    /// against the lobby a rematch "Yes" returns to, through one table of
    /// cases (own slot, the opponent's slot, battles before the rematch).
    #[test]
    fn the_character_select_lobby_matches_the_rematch_lobby() {
        for (own, opponent, battles) in [(0, 1, 1), (1, 0, 1), (0, 1, 2), (1, 0, 3), (3, 2, 1)] {
            // The rematch path: battles with an opponent who then declines; a new one joins.
            let mut rematch = Room::from_character_select();
            for _ in 0..battles {
                rematch.battle(&[own, opponent]);
            }
            rematch.member_left(opponent);
            // The character-select path, with and without the plugin; a new opponent joins.
            let native = Room::from_character_select();
            let fixed = Room::from_character_select().with_plugin(own);
            assert!(leave_offered(&rematch, own), "{own} {opponent} {battles}");
            assert!(!leave_offered(&native, own), "{own} {opponent} {battles}");
            assert_eq!(leave_offered(&fixed, own), leave_offered(&rematch, own), "{own} {opponent} {battles}");
            // The joiner's slot is the same fresh 1 in both lobbies, and no other slot is touched.
            assert_eq!(fixed.counts[opponent], rematch.counts[opponent]);
            for slot in (0..SLOTS).filter(|s| *s != own) {
                assert_eq!(fixed.counts[slot], native.counts[slot]);
            }
            // A rematch lobby with the plugin is unchanged.
            let counts = rematch.counts;
            assert_eq!(rematch.with_plugin(own).counts, counts);
        }
    }

    /// After a cancel the next character-select entry resets the room, and the
    /// plugin gives that lobby the count again; a played first match leaves the
    /// native count of one battle (candidate R puts the 1 back at the exit).
    #[test]
    fn every_character_select_entry_is_covered() {
        let mut room = Room::from_character_select().with_plugin(0);
        assert!(leave_offered(&room, 0));
        if let (_, Some((slot, count))) = frame(Some(0), QUICKPLAY, Some(0), &room.counts, true) {
            room.counts[slot] = count;
        }
        room.battle(&[0, 1]);
        assert_eq!(room.counts[0], REMATCH_COUNT);
        let room = Room::from_character_select().with_plugin(0);
        assert_eq!(room.counts[0], REMATCH_COUNT);
    }

    #[test]
    fn the_names_site_is_the_member_passs_count_load() {
        // mov w9, #0xee34; movk w9, #4, lsl #16; ldr w8, [x8, x9]; cmp w8, #1; b.hi
        let gate = GATE.stock();
        assert_eq!(gate[0], 0x529d_c689);
        assert_eq!(gate[1], 0x72a0_0089);
        let offset = ((gate[0] >> 5) & 0xffff) as usize | (((gate[1] >> 5) & 0xffff) as usize) << 16;
        assert_eq!(offset, GATE_COUNT_OFFSET);
        assert_eq!(gate[2], 0xb869_6908, "ldr w8, [x8, x9]");
        assert_eq!(((gate[2] >> 5) & 31) as usize, GATE_RECORD_REGISTER);
        assert_eq!(gate[3], 0x7100_051f, "cmp w8, #1: a count below 2 is the \"???\" rule");
        assert_eq!(gate[4] & 0xff00_001f, 0x5400_0008, "b.hi");
        assert_eq!(COUNT_OBJECT + GATE_COUNT_OFFSET, SLOT_COUNT);
        assert_eq!(GATE_OFFSET % 8, 0, "a far jump takes 4 words, all before the b.hi");
        // The names site's 64 bytes reach into the mode read it shares the function with; both stay checked.
        assert_eq!(MODE_READ.offset - GATE_OFFSET, 5 * 4);
        assert_eq!(GATE.guard[20..64], MODE_READ.guard[..44]);
    }

    #[test]
    fn opponents_show_their_names_and_nothing_is_written() {
        // An opponent the console hasn't played (count 1) or an unset count: read as 2, so the name shows.
        for (own, slot) in [(1, 0), (0, 1), (3, 2)] {
            for count in [0, FRESH_COUNT] {
                assert_eq!(shown(QUICKPLAY, slot, Some(own), count), Some(REMATCH_COUNT), "{own} {slot} {count}");
            }
            // Already 2 or more (a rematch): the stored value.
            for count in [2, 3, 7] {
                assert_eq!(shown(QUICKPLAY, slot, Some(own), count), None);
            }
            // The own slot (never compared there on Eden) and an unknown own slot: the stored value.
            assert_eq!(shown(QUICKPLAY, own, Some(own), FRESH_COUNT), None);
            assert_eq!(shown(QUICKPLAY, slot, None, FRESH_COUNT), None);
        }
        for mode in [0, 0x0701_0102, 0x0701_0400, 0x0701_0800, 0x0202_0000] {
            assert_eq!(shown(mode, 0, Some(1), FRESH_COUNT), None, "mode {mode:#x}");
        }
    }

    #[test]
    fn the_exit_request_is_the_waits_first_load() {
        // ldrb w8, [x<WAIT_CONTEXT_REGISTER>, #PRACTICE_EXIT]
        let word = WAIT.stock()[0];
        assert_eq!(word >> 22, 0x0e5, "ldrb (unsigned offset)");
        assert_eq!(((word >> 5) & 31) as usize, WAIT_CONTEXT_REGISTER);
        assert_eq!(ldst_offset(word), PRACTICE_EXIT);
        assert_eq!(word & 31, 8);
    }

    /// One console's waiting area with candidate R: `frames` waiting frames,
    /// then the exit frame. Returns the counts after it and the writes made.
    fn visit(counts: &mut [u32; SLOTS], own: usize, frames: usize) -> Vec<(usize, u32)> {
        let mut raised = None;
        let mut writes = Vec::new();
        for i in 0..=frames {
            let (next, write) = frame(raised, QUICKPLAY, Some(own), counts, i == frames);
            if let Some((slot, count)) = write {
                counts[slot] = count;
                writes.push((slot, count));
            }
            raised = next;
        }
        assert_eq!(raised, None, "nothing stays raised after the exit");
        writes
    }

    #[test]
    fn the_raised_count_lasts_only_while_the_waiting_area_is_up() {
        let mut counts = [FRESH_COUNT; SLOTS];
        // During the waiting area: raised once, so the leave is offered while the opponent loads.
        let (raised, write) = frame(None, QUICKPLAY, Some(0), &counts, false);
        assert_eq!((raised, write), (Some(0), Some((0, REMATCH_COUNT))));
        counts[0] = REMATCH_COUNT;
        assert_eq!(frame(raised, QUICKPLAY, Some(0), &counts, false), (Some(0), None));
        // The exit request: the native count goes back.
        assert_eq!(frame(raised, QUICKPLAY, Some(0), &counts, true), (None, Some((0, FRESH_COUNT))));
        // A whole visit writes 2 then 1, and nothing else.
        let mut counts = [FRESH_COUNT; SLOTS];
        assert_eq!(visit(&mut counts, 1, 30), vec![(1, REMATCH_COUNT), (1, FRESH_COUNT)]);
        assert_eq!(counts, [FRESH_COUNT; SLOTS]);
    }

    /// qcf-xfz7 m1: the search first waited in this console's own room (own slot 0, raised), then joined b's room,
    /// where slot 0 is b and this console is slot 1. The raise moves with it: b's slot gets its native 1 back, the
    /// own slot is raised, so the leave is offered as the joiner too, and both consoles count the same after it.
    #[test]
    fn the_raise_follows_this_console_into_another_room() {
        let mut counts = [FRESH_COUNT; SLOTS];
        let (raised, write) = frame(None, QUICKPLAY, Some(0), &counts, false);
        assert_eq!((raised, write), (Some(0), Some((0, REMATCH_COUNT))));
        counts[0] = REMATCH_COUNT;
        // Joined b's room: the own slot is 1 now, slot 0 (b's) still holds the plugin's 2.
        let (raised, write) = frame(raised, QUICKPLAY, Some(1), &counts, false);
        assert_eq!((raised, write), (None, Some((0, FRESH_COUNT))));
        counts[0] = FRESH_COUNT;
        let (raised, write) = frame(raised, QUICKPLAY, Some(1), &counts, false);
        assert_eq!((raised, write), (Some(1), Some((1, REMATCH_COUNT))));
        counts[1] = REMATCH_COUNT;
        // The match: the own slot's 1 goes back, then the bump; both consoles count 2 for both slots.
        let (raised, write) = frame(raised, QUICKPLAY, Some(1), &counts, true);
        assert_eq!((raised, write), (None, Some((1, FRESH_COUNT))));
        counts[1] = FRESH_COUNT;
        for slot in [0, 1] {
            counts[slot] += 1;
        }
        assert_eq!(counts[..2], [REMATCH_COUNT, REMATCH_COUNT]);
        // No own slot known for a frame (the room being rebuilt): the raise stays as it is.
        let mut counts = [FRESH_COUNT; SLOTS];
        counts[0] = REMATCH_COUNT;
        assert_eq!(frame(Some(0), QUICKPLAY, None, &counts, false), (Some(0), None));
    }

    #[test]
    fn a_rematch_lobby_is_left_alone() {
        // Native 2 or more: no raise, and no restore at the exit.
        for count in [2, 3, 5] {
            let mut counts = [FRESH_COUNT; SLOTS];
            counts[0] = count;
            assert_eq!(visit(&mut counts, 0, 10), vec![], "count {count}");
            assert_eq!(counts[0], count);
        }
        // Not Quickplay: nothing.
        let counts = [FRESH_COUNT; SLOTS];
        assert_eq!(frame(None, 0x0701_0400, Some(0), &counts, false), (None, None));
        // The slot changed under a raise (a new character-select entry): dropped, not restored.
        assert_eq!(frame(Some(0), QUICKPLAY, Some(0), &counts, false), (None, None));
        let mut counts = [FRESH_COUNT; SLOTS];
        counts[0] = 3;
        assert_eq!(frame(Some(0), QUICKPLAY, Some(0), &counts, true), (None, None));
    }

    /// Parity with the game: after the first match from the character select
    /// and a rematch, the plugin's console counts its own slot exactly as the
    /// opponent's console counts it, and as a console without the plugin
    /// counts its own (qcf-srm11: 1, 2, 3; v0.2.0 in qcf-srm12: 2, 3 against
    /// the opponent's 2).
    #[test]
    fn both_consoles_count_the_same_after_each_battle() {
        for (own, opponent) in [(0, 1), (1, 0), (3, 2)] {
            // The fixed console (a), a vanilla one in its place (v), and the opponent's view (b).
            let mut a = [FRESH_COUNT; SLOTS];
            let mut v = [FRESH_COUNT; SLOTS];
            let mut b = [FRESH_COUNT; SLOTS];
            for battle in 1..=3u32 {
                visit(&mut a, own, 20);
                for counts in [&mut a, &mut v, &mut b] {
                    counts[own] += 1;
                    counts[opponent] += 1;
                }
                assert_eq!(a[own], b[own], "battle {battle}: a's own slot as a and b count it");
                assert_eq!(a, v, "battle {battle}: a counts as a console without the plugin");
                assert_eq!(a[own], FRESH_COUNT + battle);
            }
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
