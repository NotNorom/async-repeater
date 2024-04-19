use std::time::Duration;

use async_repeater::RepeaterEntry;

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

    fn when(&self) -> Duration {
        self.interval
    }

    fn key(&self) -> Self::Key {
        self.id
    }
    
    fn reset_delay(&mut self) {}
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let callback = |entry, _| async move { println!("Popped {entry:?}") };

    // create and start the repeater
    let handle = async_repeater::Repeater::with_capacity(25).run_with_async_callback(callback);

    // create entries with the same interval
    let dur = Duration::from_secs(5);
    let repetition = vec![Entry::new(0, dur), Entry::new(1, dur), Entry::new(2, dur)];

    // insert the entries into the repeater with 1 second delay in between
    for repetition in repetition {
        handle.insert(repetition).await.unwrap();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // wait for a ctrl-c.
    // If this wasn't there, the program would exit immidiatly without running
    // a single repetition
    tokio::signal::ctrl_c().await.unwrap();
    handle.stop().await.unwrap();
    println!("Stopping cycler");
}
