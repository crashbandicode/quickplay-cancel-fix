//! Switch half: the version and site checks, the four inline hooks that drive
//! `model::Gate` (on other plugins' hooks there if it finds them, see
//! `chain`), the game's own member-state setter at the end of a hold, and the
//! install line. The hooks run on the game's main thread and allocate
//! nothing; the diagnostic build formats events into a static ring that a
//! writer thread appends to `events.log`.

use crate::chain::{self, hook::Outcome, Entry};
use crate::model::{self, Gate, Kind, Line, Note, Site, Tick};
use core::cell::UnsafeCell;
use core::fmt::Write as _;
use skyline::hooks::{getRegionAddress, InlineCtx, Region};
use skyline::nn;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

const DATA_DIR: &str = "sd:/ultimate/quickplay-cancel-fix";
const OPEN_WRITE_APPEND: i32 = 2 | 4;

/// Counts calls into the heap. Nothing in the plugin is meant to make one.
struct CountingAllocator;
static HEAP_ALLOCATIONS: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        HEAP_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        System.alloc(layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        HEAP_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        System.alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        HEAP_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        System.realloc(ptr, layout, size)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

static TEXT_BASE: AtomicU64 = AtomicU64::new(0);

/// The first-match hold. Only the hooks touch it, all on the game's main
/// thread; `BUSY` turns a re-entry (none is expected) into a no-op.
struct Shared(UnsafeCell<Gate>);
// SAFETY: accessed only inside `with_gate`, which `BUSY` makes exclusive.
unsafe impl Sync for Shared {}
static GATE: Shared = Shared(UnsafeCell::new(Gate::new()));
static BUSY: AtomicBool = AtomicBool::new(false);

fn with_gate<R>(f: impl FnOnce(&mut Gate) -> R) -> Option<R> {
    if BUSY
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        return None;
    }
    // SAFETY: `BUSY` is held.
    let result = f(unsafe { &mut *GATE.0.get() });
    BUSY.store(false, Ordering::Release);
    Some(result)
}

fn tick() -> u64 {
    unsafe { nn::os::GetSystemTick() as u64 }
}

fn base() -> u64 {
    TEXT_BASE.load(Ordering::Relaxed)
}

unsafe fn read_u32(address: u64) -> u32 {
    (address as *const u32).read_unaligned()
}

unsafe fn read_u64(address: u64) -> u64 {
    (address as *const u64).read_unaligned()
}

/// The pointer stored at `address`, if `address` and it are non-null.
unsafe fn ptr(address: u64) -> Option<u64> {
    if address == 0 {
        return None;
    }
    Some(read_u64(address)).filter(|p| *p != 0)
}

unsafe fn game<F: Copy>(offset: usize) -> F {
    let code = (base() as usize + offset) as *const ();
    core::mem::transmute_copy(&code)
}

/// The online service, as `.151b030..151b03c` loads it.
unsafe fn service() -> Option<u64> {
    ptr(base() + model::SERVICE as u64)
        .and_then(|holder| ptr(holder + 8))
        .and_then(|inner| ptr(inner + model::SERVICE_SLOT as u64))
}

#[cfg(feature = "diag")]
fn note(note: Option<Note>, detail: u64) {
    if let Some(note) = note {
        diag::push(format_args!("diag {note:?} detail={detail:#x}"));
    }
}

#[cfg(not(feature = "diag"))]
fn note(_: Option<Note>, _: u64) {}

/// The Quickplay sequence's search start (`model::SEARCH`): x8 = the service,
/// before `.16b4920` sets its matchmaking mode.
unsafe extern "C" fn search_start(ctx: &mut InlineCtx) {
    let svc = ctx.registers[8].x();
    if svc == 0 {
        return;
    }
    let mode = read_u32(svc + model::SVC_MODE as u64);
    let result = with_gate(|gate| gate.search(mode)).flatten();
    note(result, u64::from(mode));
}

/// The waiting area's ready report (`model::READY`): x0 = the service,
/// w1 = the state.
unsafe extern "C" fn ready_report(ctx: &mut InlineCtx) {
    let state = ctx.registers[1].w();
    if state != model::READY_STATE {
        return;
    }
    let svc = ctx.registers[0].x();
    if svc == 0 {
        return;
    }
    let role = model::role(
        read_u64(svc + model::SVC_MY_ID as u64),
        read_u64(svc + model::SVC_HOST_ID as u64),
    );
    let online = model::online(read_u32(base() + model::MODE as u64));
    if let Some((report, result)) = with_gate(|gate| gate.ready(state, role, online)) {
        if report != state {
            ctx.registers[1].set_w(report);
        }
        note(result, u64::from(report));
    }
}

/// A frame of the wait loop on a console that isn't the room's host
/// (`model::WAIT`): ends the hold after the full-room wait with the game's
/// own setter.
unsafe extern "C" fn wait_frame(_: &mut InlineCtx) {
    let Some(svc) = service() else {
        return;
    };
    let round_started = read_u32(svc + model::SVC_ROUND_MASK as u64) != 0;
    let holding = with_gate(|gate| gate.holding()).unwrap_or(false);
    if !holding {
        return;
    }
    let input_free: extern "C" fn() -> u32 = game(model::INPUT_FREE_OFFSET);
    let input_free = input_free() & 1 != 0;
    let Some((step, result)) = with_gate(|gate| gate.tick(round_started, input_free)) else {
        return;
    };
    match step {
        Tick::Release => {
            let set_state: extern "C" fn(u64, u32) -> u32 = game(model::SET_STATE_OFFSET);
            let reported = set_state(svc, model::READY_STATE) & 1;
            note(result, u64::from(reported));
        }
        #[cfg(feature = "diag")]
        Tick::Holding(frames) if frames % 60 == 0 => {
            diag::push(format_args!("diag holding frames={frames}"));
        }
        _ => note(result, 0),
    }
}

/// PracticeScene's leave state on its first frame (`model::LEAVE`): Leave
/// Battle was confirmed, so a hold ends.
unsafe extern "C" fn leave_confirmed(_: &mut InlineCtx) {
    let result = with_gate(|gate| gate.leave()).flatten();
    note(result, 0);
}

type Path = Line<96>;

fn path(args: core::fmt::Arguments) -> Path {
    let mut path = Path::new();
    let _ = path.write_fmt(args);
    path
}

/// An `nn::fs` file opened to append; closed on drop. A closed file is on
/// the card at once, while Eden keeps an open file's writes until the host
/// file is closed (its `FlushFile` does not reach the host).
struct NxFile {
    handle: nn::fs::FileHandle,
    offset: i64,
}

impl NxFile {
    fn append(path: &mut Path) -> Option<Self> {
        let path = path.with_nul()?.as_ptr();
        unsafe {
            // Fails harmlessly when the file exists.
            nn::fs::CreateFile(path, 0);
            let mut handle = nn::fs::FileHandle { handle: 0 };
            if nn::fs::OpenFile(&mut handle, path, OPEN_WRITE_APPEND) != 0 {
                return None;
            }
            let mut offset = 0;
            if nn::fs::GetFileSize(&mut offset, handle) != 0 {
                nn::fs::CloseFile(handle);
                return None;
            }
            Some(Self { handle, offset })
        }
    }

    fn write(&mut self, bytes: &[u8]) -> bool {
        let option = nn::fs::WriteOption { flags: 0 };
        let written = unsafe {
            nn::fs::WriteFile(
                self.handle,
                self.offset,
                bytes.as_ptr(),
                bytes.len() as u64,
                &option,
            )
        } == 0;
        if written {
            self.offset += bytes.len() as i64;
        }
        written
    }
}

impl Drop for NxFile {
    fn drop(&mut self) {
        unsafe {
            nn::fs::FlushFile(self.handle);
            nn::fs::CloseFile(self.handle);
        }
    }
}

fn append_log(bytes: &[u8]) {
    if let Some(mut log) = NxFile::append(&mut path(format_args!("{DATA_DIR}/events.log"))) {
        log.write(bytes);
    }
}

/// Eden-only diagnostics: an event ring the hooks format into, and a writer
/// thread that appends it to `events.log`.
#[cfg(feature = "diag")]
mod diag {
    use super::*;
    use std::sync::atomic::fence;

    const EVENT_RING: usize = 32;
    const EVENT_LEN: usize = 160;
    const WRITER_STACK: usize = 16 * 1024;

    /// Shared cell whose accesses are ordered by the slot's `seq`.
    struct Owned<T>(UnsafeCell<T>);
    // SAFETY: see `push` and `drain`.
    unsafe impl<T> Sync for Owned<T> {}

    /// One formatted line; `seq` is the event index + 1 once it is complete.
    struct EventSlot {
        seq: AtomicU64,
        text: Owned<Line<EVENT_LEN>>,
    }

    static EVENTS: [EventSlot; EVENT_RING] = [const {
        EventSlot {
            seq: AtomicU64::new(0),
            text: Owned(UnsafeCell::new(Line::new())),
        }
    }; EVENT_RING];
    static EVENT_HEAD: AtomicU64 = AtomicU64::new(0);

    /// The writer thread's object and stack, static so no heap is involved.
    #[repr(C, align(4096))]
    struct ThreadStack(UnsafeCell<[u8; WRITER_STACK]>);
    #[repr(C, align(16))]
    struct ThreadObject(UnsafeCell<[u8; 0x1c0]>);
    // SAFETY: handed to nn::os once, at startup, and never touched again.
    unsafe impl Sync for ThreadStack {}
    unsafe impl Sync for ThreadObject {}
    static WRITER_STACK_MEMORY: ThreadStack = ThreadStack(UnsafeCell::new([0; WRITER_STACK]));
    static WRITER_THREAD: ThreadObject = ThreadObject(UnsafeCell::new([0; 0x1c0]));

    pub fn push(args: core::fmt::Arguments) {
        let index = EVENT_HEAD.fetch_add(1, Ordering::AcqRel);
        let slot = &EVENTS[index as usize % EVENT_RING];
        slot.seq.store(0, Ordering::Release);
        // SAFETY: this index owns the slot until it publishes `seq`; the
        // writer re-checks `seq` after copying and drops an overwritten line.
        let text = unsafe { &mut *slot.text.0.get() };
        text.clear();
        let _ = write!(text, "{} ", tick());
        let _ = text.write_fmt(args);
        let _ = text.write_char('\n');
        slot.seq.store(index + 1, Ordering::Release);
    }

    /// Copies every published event after `tail` into `events.log`.
    fn drain(tail: &mut u64) {
        let head = EVENT_HEAD.load(Ordering::Acquire);
        if head == *tail {
            return;
        }
        if head - *tail > EVENT_RING as u64 {
            *tail = head - EVENT_RING as u64;
        }
        let Some(mut file) = NxFile::append(&mut path(format_args!("{DATA_DIR}/events.log")))
        else {
            return;
        };
        while *tail < head {
            let slot = &EVENTS[*tail as usize % EVENT_RING];
            let seq = slot.seq.load(Ordering::Acquire);
            if seq == 0 || seq < *tail + 1 {
                break; // still being written
            }
            let mut copy = Line::<EVENT_LEN>::new();
            // SAFETY: read under the sequence check below.
            let text = unsafe { &*slot.text.0.get() };
            let _ = copy.write_str(core::str::from_utf8(text.as_bytes()).unwrap_or(""));
            fence(Ordering::Acquire);
            if seq == *tail + 1 && slot.seq.load(Ordering::Relaxed) == seq {
                file.write(copy.as_bytes());
            }
            *tail += 1;
        }
    }

    extern "C" fn writer_main(_: *mut skyline::libc::c_void) {
        let mut tail = 0;
        loop {
            unsafe {
                nn::os::SleepThread(nn::TimeSpan {
                    nanoseconds: 200_000_000,
                })
            };
            drain(&mut tail);
        }
    }

    pub fn start_writer() -> bool {
        unsafe {
            let thread = WRITER_THREAD.0.get().cast::<nn::os::ThreadType>();
            let priority = nn::os::GetThreadPriority(nn::os::GetCurrentThread());
            let created = nn::os::CreateThread(
                thread,
                writer_main,
                core::ptr::null_mut(),
                WRITER_STACK_MEMORY.0.get().cast::<u8>(),
                WRITER_STACK,
                priority,
            );
            if created != 0 {
                return false;
            }
            nn::os::SetThreadNamePointer(thread, c"quickplay-cancel-fix".as_ptr().cast());
            nn::os::StartThread(thread);
            true
        }
    }
}

/// The site's address, when its 64 bytes are in `.text`.
unsafe fn site_address(site: &Site) -> Option<u64> {
    let base = getRegionAddress(Region::Text) as u64;
    let end = getRegionAddress(Region::Rodata) as u64;
    let address = base + site.offset as u64;
    (address.is_multiple_of(4) && address + 64 <= end).then_some(address)
}

/// The site's instructions now, with its address.
unsafe fn site_words(site: &Site) -> Option<(u64, [u32; chain::WORDS])> {
    site_address(site).map(|address| (address, chain::hook::words_at(address)))
}

/// Whether the plugin can use the site as it is now (see `Site::usable`).
unsafe fn usable(site: &Site) -> bool {
    site_words(site).is_some_and(|(address, words)| site.usable(&words, address))
}

/// Another plugin's hook is on a site this plugin calls.
unsafe fn other_hook(site: &Site) -> bool {
    site.kind != Kind::Hooked && site_words(site).is_some_and(|(_, words)| words != site.stock())
}

/// Hooks `site`, on another plugin's hook there if it finds one. A near-only
/// site that got a far jump is put back.
unsafe fn hook(site: &Site, callback: unsafe extern "C" fn(&mut InlineCtx)) -> (Entry, Outcome) {
    let Some((address, before)) = site_words(site) else {
        return (Entry::Foreign, Outcome::Skipped);
    };
    if !site.usable(&before, address) {
        return (Entry::Foreign, Outcome::Skipped);
    }
    let (entry, outcome) = chain::hook::inline(address, &site.stock(), callback);
    if site.near_only && outcome == Outcome::Far {
        chain::hook::restore(address, &before);
        return (entry, Outcome::Restored);
    }
    (entry, outcome)
}

fn version_is_supported() -> bool {
    let mut version = nn::oe::DisplayVersion { name: [0; 16] };
    unsafe { nn::oe::GetDisplayVersion(&mut version) };
    let end = version.name.iter().position(|v| *v == 0).unwrap_or(16);
    version.name[..end] == *model::SUPPORTED_VERSION.as_bytes()
}

/// Appends the install line to `events.log` and the Skyline log.
fn report_install(report: &mut Line<320>) {
    let _ = write!(
        report,
        " heap_allocations={}",
        HEAP_ALLOCATIONS.load(Ordering::Relaxed)
    );
    skyline::logging::log(core::str::from_utf8(report.as_bytes()).unwrap_or(""));
    let _ = report.write_char('\n');
    for directory in ["sd:/ultimate", DATA_DIR] {
        if let Some(directory) = path(format_args!("{directory}")).with_nul() {
            unsafe { nn::fs::CreateDirectory(directory.as_ptr()) };
        }
    }
    append_log(report.as_bytes());
}

#[skyline::main(name = "quickplay-cancel-fix")]
pub fn main() {
    let mut report = Line::<320>::new();
    let _ = write!(
        report,
        "{} plugin=quickplay-cancel-fix/{}",
        tick(),
        env!("CARGO_PKG_VERSION")
    );
    #[cfg(feature = "diag")]
    let _ = write!(report, " build=diag");
    if !version_is_supported() {
        let _ = write!(report, " result=unsupported_version");
        report_install(&mut report);
        return;
    }
    TEXT_BASE.store(
        unsafe { getRegionAddress(Region::Text) as u64 },
        Ordering::Relaxed,
    );
    let result = match model::SITES.iter().find(|site| !unsafe { usable(site) }) {
        Some(site) => {
            let _ = write!(report, " site={}", site.name);
            "not_installed"
        }
        None => {
            // The hold's release (wait), its drop (leave) and the arming
            // (search) go in first; the ready report that starts a hold only
            // if all three did.
            let mut all = true;
            let hooks: [(&Site, unsafe extern "C" fn(&mut InlineCtx)); 4] = [
                (&model::WAIT, wait_frame),
                (&model::LEAVE, leave_confirmed),
                (&model::SEARCH, search_start),
                (&model::READY, ready_report),
            ];
            for (site, callback) in hooks {
                if !all {
                    let _ = write!(report, " {}=not_tried", site.name);
                    continue;
                }
                let (entry, outcome) = unsafe { hook(site, callback) };
                let _ = write!(report, " {}={}/{}", site.name, entry.name(), outcome.name());
                all = outcome.hooked();
            }
            if all {
                "installed"
            } else {
                "not_installed"
            }
        }
    };
    for site in model::SITES {
        if unsafe { other_hook(site) } {
            let _ = write!(report, " other_hook={}", site.name);
        }
    }
    let _ = write!(report, " result={result}");
    report_install(&mut report);
    #[cfg(feature = "diag")]
    if !diag::start_writer() {
        report.clear();
        let _ = write!(report, "{} writer=not_started", tick());
        report_install(&mut report);
    }
}
