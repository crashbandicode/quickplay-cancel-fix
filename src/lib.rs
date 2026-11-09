//! quickplay-cancel-fix: in Smash Ultimate 13.0.5 Quickplay, the first match
//! after entering the character select can be left before it loads, as later
//! matches can. When this console joins that first room, its waiting area
//! reports ready after the same 180-frame wait an earlier arrival gets from
//! the host, instead of at once; the battle start itself stays the game's own.
//! Every later match is untouched.
pub mod chain;
pub mod model;
#[cfg(target_os = "switch")]
mod runtime;
