//! This example uses the stream implementation on the DelayQueue directly.
//! It skips the handle

use async_repeater::{Repeater, RepeaterEntry};
use std::time::Duration;
use tokio_stream::StreamExt;

/// Most basic struct for this to work.
/// Only carries an id and an interval
#[derive(Debug, Clone)]
pub struct Entry {
    id: u8,
    interval: Duration,
}

impl Entry {
    pub fn new(id: u8, interval: Duration) -> Self {
        Self { id, interval }
    }
}

impl RepeaterEntry for Entry {
    type Key = u8;

    fn interval(&self) -> Duration {
        self.interval
    }

    fn key(&self) -> Self::Key {
        self.id
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // create but not start the repeater
    let mut repeater = Repeater::<Entry>::with_capacity(7);

    // create entries as a stream
    let repetitions = tokio_stream::iter(vec![
        Entry::new(0, Duration::from_millis(3000)),
        Entry::new(1, Duration::from_millis(1000)),
        Entry::new(2, Duration::from_millis(2000)),
    ]);
    tokio::pin!(repetitions);

    // select either an element from the stream or
    // an element from the repeater or
    // from the ctrl-c signal
    loop {
        tokio::select! {
            Some(item) = repeater.next() => {
                println!("Popped: {item:?}");
                repeater.insert(item);
            }
            Some(item) = repetitions.next() => {
                repeater.insert(item);
            }
            _ = tokio::signal::ctrl_c() => {
                break
            }
        }
    }
}
