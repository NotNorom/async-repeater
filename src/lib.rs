mod entry;
mod handle;
mod repeater;

use std::time::{Duration, Instant};

pub use entry::RepeaterEntry;
pub use handle::RepeaterHandle;
pub use repeater::Repeater;

/// For defining a delay
#[derive(Debug, Clone, Copy)]
pub enum Delay {
    Relative(Duration),
    Absolute(Instant),
    None,
}

impl From<Delay> for Duration {
    fn from(value: Delay) -> Self {
        match value {
            Delay::Relative(dur) => dur,
            Delay::Absolute(inst) => inst.duration_since(Instant::now()),
            Delay::None => Duration::ZERO,
        }
    }
}
