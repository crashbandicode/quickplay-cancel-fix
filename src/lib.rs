//! quickplay-cancel-fix: in Smash Ultimate 13.0.5 Quickplay, the first match
//! after the character select can be left while the opponent loads, as it can
//! after a rematch "Yes": Pause, Leave Battle, back to the character select.
//! The lobby entered from the character select takes the rematch lobby's
//! battle count for this console's slot; the leave itself is the game's own.
pub mod chain;
pub mod hold;
pub mod model;
#[cfg(target_os = "switch")]
mod runtime;
