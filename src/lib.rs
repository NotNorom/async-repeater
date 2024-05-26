mod entry;
mod handle;
mod repeater;

use std::time::{Duration, SystemTime, SystemTimeError};

pub use entry::RepeaterEntry;
pub use handle::RepeaterHandle;
pub use repeater::Repeater;

/// For defining a delay
#[derive(Debug, Clone, Copy)]
pub enum Delay {
    Relative(Duration),
    Absolute(SystemTime),
    None,
}

impl TryFrom<Delay> for Duration {
    type Error = SystemTimeError;

    fn try_from(value: Delay) -> Result<Duration, SystemTimeError> {
        match value {
            Delay::Relative(dur) => Ok(dur),
            Delay::Absolute(timestamp) => timestamp.duration_since(SystemTime::now()),
            Delay::None => Ok(Duration::ZERO),
        }
    }
}
