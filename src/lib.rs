//! quickplay-cancel-fix: in Smash Ultimate 13.0.5 online rooms, a member who
//! joins after the host's room countdown has run out gets the same short wait
//! before the battle starts as one who joins earlier, so either player can
//! still leave the waiting area (Leave Battle) before the match loads. The
//! battle start itself stays the game's own.
pub mod chain;
pub mod model;
#[cfg(target_os = "switch")]
mod runtime;
