use crate::{
    Delay, RepeaterEntry,
    handle::{Message, RepeaterHandle},
};
use futures_core::ready;
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, SystemTime},
};
use tokio::sync::mpsc::{Receiver, channel};
use tokio_stream::{self, Stream, StreamExt};
use tokio_util::time::{DelayQueue, delay_queue};

/// Repeats structs after user defined Duration.
pub struct Repeater<E>
where
    E: RepeaterEntry + Clone + Unpin + Send,
{
    entries: HashMap<E::Key, (E, delay_queue::Key)>,
    queue: DelayQueue<E::Key>,
}

impl<E> Repeater<E>
where
    E: RepeaterEntry + Clone + Unpin + Send + Sync,
{
    /// Create a new repeater with given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            queue: DelayQueue::with_capacity(capacity),
        }
    }

    /// Insert entry into repeater
    ///
    /// If entry with same key already exists, it will be replaced.
    /// Replacement will also reset the repetition.
    ///
    /// Insertion respects the delay() call
    pub fn insert(&mut self, e: E) {
        // if there is a delay before the first run
        let interval = match e.delay() {
            Delay::Relative(dur) => dur,
            Delay::Absolute(inst) => inst.duration_since(SystemTime::now()).unwrap_or_else(|_| e.interval()),
            Delay::None => Duration::default(),
        };

        match self.entries.get_mut(&e.key()) {
            Some((current_item, queue_key)) => {
                // if an item with the same key already exists,
                // then replace that item with the new one and set the timer to the new value as well
                self.queue.reset(queue_key, interval);
                *current_item = e;
            }
            _ => {
                // if the item is new, insert it and the time at which to start
                let queue_key = self.queue.insert(e.key(), interval);
                self.entries.insert(e.key(), (e.clone(), queue_key));
            }
        }
    }

    /// Remove entry with key from repeater
    pub fn remove(&mut self, key: &E::Key) {
        let Some((_, key)) = self.entries.remove(key) else {
            return;
        };
        self.queue.remove(&key);
    }

    /// Clear the repeater. This removes all entries.
    pub fn clear(&mut self) {
        self.queue.clear();
        self.entries.clear();
    }

    /// Number of entries in the repeater
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Return true if repeater is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Starts the repeater in a background task
    ///
    /// This consumes the repeater and returns a [`RepeaterHandle`] which allows communication with
    /// the background task.
    pub fn run_with_async_callback<F, Fut>(self, callback: F) -> RepeaterHandle<E>
    where
        F: FnOnce(E, RepeaterHandle<E>) -> Fut + Send + 'static + Clone,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let (tx, rx) = channel(self.queue.capacity());
        let handle = RepeaterHandle::new(tx);

        tokio::spawn(self.handle_messages(rx, handle.clone(), callback));

        handle
    }

    /// Handle communication from the [`RepeaterHandle`].
    ///
    /// It's mainly a separate function to decrease indention inside [`Self::run_with_async_callback`].
    /// This is run in a separate task
    async fn handle_messages<F, Fut>(mut self, mut rx: Receiver<Message<E>>, handle: RepeaterHandle<E>, callback: F)
    where
        F: FnOnce(E, RepeaterHandle<E>) -> Fut + Send + 'static + Clone,
        Fut: Future<Output = ()> + Send + 'static,
    {
        loop {
            let cb = callback.clone();
            let handle = handle.clone();

            tokio::select! {
                Some(message) = rx.recv() => {
                    match message {
                        Message::Insert(entry) => self.insert(entry.clone()),
                        Message::Remove(key) => self.remove(&key),
                        Message::Clear => { self.clear()},
                        Message::Stop => { break },
                        Message::Len(reply_sender) => { let _ = reply_sender.send(self.len()); }
                        Message::IsEmtpy(reply_sender) => { let _ = reply_sender.send(self.is_empty()); }
                    }
                }
                Some(entry) = self.next() => {
                    tokio::spawn((cb)(entry, handle));
                }
            }
        }
    }

    /// Poll for a new item.
    ///
    /// If an item is available it will also be re-inserted.
    /// Re-insertions ignore the delay and will only respect the interval.
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<E>> {
        let entry_id: Option<E::Key> = ready!(self.queue.poll_expired(cx)).map(delay_queue::Expired::into_inner);
        if let Some(entry_id) = entry_id {
            let (entry, queue_key) = self.entries.get_mut(&entry_id).unwrap();

            let new_queue_key = self.queue.insert(entry.key(), entry.interval());
            *queue_key = new_queue_key;

            return Poll::Ready(Some(entry.clone()));
        }

        Poll::Pending
    }
}

impl<E> Stream for Repeater<E>
where
    E: RepeaterEntry + Clone + Unpin + Send + Sync,
{
    type Item = E;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<<Self as Stream>::Item>> {
        Repeater::poll_next(self.get_mut(), cx)
    }
}
