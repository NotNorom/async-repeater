use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

use crate::Delay;

/// Entry for the Repeater
pub trait RepeaterEntry {
    /// The type of key
    type Key: Hash + Eq + Unpin + Send + Debug;

    /// Duration after which the entry should be repeated
    fn interval(&self) -> Duration;

    /// Duration after which the first repetition is run
    ///
    /// An example: It's 2:57pm and we want to start a repetition with an interval of 60 seconds,
    /// but also want to wait until it's exactly 3pm.
    /// We'd either set the delay to be relative 180 seconds or specify the absolute unix time stamp.
    ///
    /// If not implemented, it returns Delay::None starting the repetition instantly
    fn delay(&self) -> Delay {
        Delay::None
    }

    /// To identify the entry.
    ///
    /// We don't want to store the whole entry in the underlying queue,
    /// but only a key
    fn key(&self) -> Self::Key;
}
