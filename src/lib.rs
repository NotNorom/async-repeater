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
