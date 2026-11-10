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

use crate::chain;

pub const SUPPORTED_VERSION: &str = "13.0.5";

/// How a site is checked before anything is installed. Other plugins' hooks
/// are accepted as `chain` reads them; any other change is not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// One of the plugin's inline hooks goes here: another plugin's hook on it
    /// is chained onto (`chain::entry`).
    Hooked,
    /// Code the plugin relies on (it reads or writes the data this code
    /// reads or writes): other plugins' hooks in it are fine.
    Relied,
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
            Kind::Relied => chain::usable(&stock, words, address, 0),
        }
    }
}

/// PracticeScene's running state (`.1851660`, every waiting-area frame), after
/// its update call: `ldrb w8, [x19, #0xf0]` (its exit request), x19 = the
/// PracticeScene context. Nothing jumps into the 20 bytes from here; no flags
/// are live.
pub const WAIT_OFFSET: usize = 0x1851674;
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

/// Every site checked before anything is installed.
pub const SITES: [&Site; 5] = [&WAIT, &COUNT_RESET, &COUNT_BUMP, &STATION_OF, &MODE_READ];

/// The main-image offsets this plugin hooks.
pub const HOOKED_SITES: [usize; 1] = [WAIT_OFFSET];

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

/// The count to write into this console's slot of a Quickplay lobby, if any:
/// a character-select entry's fresh 1 becomes the rematch path's 2; anything
/// else (a rematch lobby's 2 or more, an unset 0) stays.
pub fn adopt(mode: u32, count: u32) -> Option<u32> {
    (mode == QUICKPLAY && count == FRESH_COUNT).then_some(REMATCH_COUNT)
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
    }

    /// Where each site would be with the image at a page-aligned base.
    fn at(site: &Site) -> u64 {
        0x80_0000_0000 + site.offset as u64
    }

    #[test]
    fn sites_are_checked_by_kind() {
        let hooked: Vec<_> = SITES.iter().filter(|s| s.kind == Kind::Hooked).map(|s| s.offset).collect();
        assert_eq!(hooked, HOOKED_SITES);
        for site in SITES {
            assert!(site.usable(&site.stock(), at(site)), "{}", site.name);
        }
        // TrainingRooms fingerprints .236b470+0x104, .1851480+0x100, .18512c0+0x50 and .1851310+0x30: the hook
        // is outside them.
        for (start, len) in [(0x236b470, 0x104), (0x1851480, 0x100), (0x18512c0, 0x50), (0x1851310, 0x30)] {
            assert!(WAIT_OFFSET + 64 <= start || WAIT_OFFSET >= start + len);
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
