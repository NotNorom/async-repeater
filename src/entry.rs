use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

/// Entry for the Repeater
pub trait RepeaterEntry {
    /// The type of key
    type Key: Hash + Eq + Unpin + Send + Debug;

    /// Duration after which the entry should be repeated
    fn when(&self) -> Duration;
    /// Duration after which the first repetition should start.
    /// 
    /// If not implemented, the first repetition starts instantly.
    fn delay(&self) -> Duration {
        Duration::default()
    }
    /// To identify the entry.
    /// We don't want to store the whole entry in the underlying queue,
    /// but only a key
    fn key(&self) -> Self::Key;
}
