use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

use crate::Delay;

/// Entry for the Repeater
pub trait RepeaterEntry {
    /// The type of key
    type Key: Hash + Eq + Unpin + Send + Debug;

    /// Duration after which the entry should be repeated
    fn when(&self) -> Duration;

    /// Duration after which the first repetition should start.
    ///
    /// If not implemented, the first repetition starts instantly.
    fn delay(&self) -> Delay {
        Delay::None
    }

    /// Reset the delay
    /// After calling this function, the next call to [`RepeaterEntry::delay`] should return `None`.
    fn reset_delay(&mut self);

    /// To identify the entry.
    /// We don't want to store the whole entry in the underlying queue,
    /// but only a key
    fn key(&self) -> Self::Key;
}
