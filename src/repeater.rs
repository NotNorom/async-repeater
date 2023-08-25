use crate::{
    handle::{Message, RepeaterHandle},
    RepeaterEntry,
};
use futures_core::ready;
use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::mpsc::{channel, Receiver};
use tokio_stream::{self, Stream, StreamExt};
use tokio_util::time::{delay_queue, DelayQueue};

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
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(capacity),
            queue: DelayQueue::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, e: E) {
        if let Some((current_item, queue_key)) = self.entries.get_mut(&e.key()) {
            self.queue.reset(queue_key, e.when());
            *current_item = e;
        } else {
            let queue_key = self.queue.insert(e.key(), e.when());
            self.entries.insert(e.key(), (e.clone(), queue_key));
        }
    }

    pub fn remove(&mut self, key: &E::Key) {
        let Some((_, key)) = self.entries.remove(key) else { return };
        self.queue.remove(&key);
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub async fn run_with_async_callback<F, Fut>(self, callback: F) -> RepeaterHandle<E>
    where
        F: FnMut(E) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let (tx, rx) = channel(128);
        tokio::spawn(self.handle_messages(rx, callback));

        RepeaterHandle::new(tx)
    }

    /// Separate function to decrease indention level a bit
    async fn handle_messages<F, Fut>(mut self, mut rx: Receiver<Message<E>>, mut callback: F) -> ()
    where
        F: FnMut(E) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send,
    {
        loop {
            tokio::select! {
                Some(message) = rx.recv() => {
                    match message {
                        Message::Insert(entry) => self.insert(entry),
                        Message::Remove(key) => self.remove(&key),
                        Message::Clear => { self.clear()},
                        Message::Stop => { break },
                    }
                }
                Some(entry) = self.next() => {
                    // rescheduling before running the callback to be more accurate to the time
                    self.insert(entry.clone());
                    callback(entry).await;
                }
            }
        }
    }

    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<E>> {
        let entry_id: Option<E::Key> =
            ready!(self.queue.poll_expired(cx)).map(|entry| entry.into_inner());
        if let Some(entry_id) = entry_id {
            let (entry, _) = self.entries.remove(&entry_id).unwrap();
            return Poll::Ready(Some(entry));
        }

        Poll::Pending
    }
}

impl<E> Stream for Repeater<E>
where
    E: RepeaterEntry + Clone + Unpin + Send + Sync,
{
    type Item = E;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<<Self as Stream>::Item>> {
        Repeater::poll_next(self.get_mut(), cx)
    }
}
