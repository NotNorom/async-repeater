use std::fmt::Debug;
use std::hash::Hash;
use std::time::Duration;

pub trait RepeaterEntry {
    type Key: Hash + Eq + Unpin + Send + Debug;

    fn when(&self) -> Duration;
    fn delay(&self) -> Duration {
        Duration::default()
    }
    fn key(&self) -> Self::Key;
}
