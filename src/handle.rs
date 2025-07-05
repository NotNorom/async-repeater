use std::fmt::Display;

use tokio::sync::{
    mpsc::{self},
    oneshot::{self, channel},
};

use crate::entry::RepeaterEntry;

/// Internal messaage for communication between [`RepeaterHandle`](RepeaterHandle) and [`Repeater`](crate::Repeater)
pub(crate) enum Message<E>
where
    E: RepeaterEntry + Clone + Unpin + 'static,
{
    Insert(E),
    Remove(E::Key),
    Clear,
    Stop,
    Len(oneshot::Sender<usize>),
    IsEmtpy(oneshot::Sender<bool>),
}

/// Used to communicate with the repeater task
///
/// Returned after the call to [`Repeater::run_with_async_callback`](crate::Repeater::run_with_async_callback)
#[derive(Debug, Clone)]
pub struct RepeaterHandle<E>
where
    E: RepeaterEntry + Clone + Unpin + 'static,
{
    tx: mpsc::Sender<Message<E>>,
}

impl<E> RepeaterHandle<E>
where
    E: RepeaterEntry + Clone + Unpin + 'static,
{
    pub(crate) fn new(sender: mpsc::Sender<Message<E>>) -> Self {
        Self { tx: sender }
    }

    /// Insert entry into [`Repeater`](crate::Repeater)
    pub async fn insert(&self, e: E) -> Result<(), ConnectionLost> {
        Ok(self.tx.send(Message::Insert(e)).await?)
    }

    /// Remove entry from [`Repeater`](crate::Repeater)
    pub async fn remove(&self, key: E::Key) -> Result<(), ConnectionLost> {
        Ok(self.tx.send(Message::Remove(key)).await?)
    }

    /// Remove all entries from [`Repeater`](crate::Repeater)
    pub async fn clear(&self) -> Result<(), ConnectionLost> {
        Ok(self.tx.send(Message::Clear).await?)
    }

    /// Stop a [`Repeater`](crate::Repeater)
    pub async fn stop(self) -> Result<(), ConnectionLost> {
        Ok(self.tx.send(Message::Stop).await?)
    }

    /// Return the number of entries in [`Repeater`](crate::Repeater)
    pub async fn len(self) -> Result<usize, ConnectionLost> {
        let (reply_tx, reply_rx) = channel();

        self.tx.send(Message::Len(reply_tx)).await?;
        Ok(reply_rx.await?)
    }

    /// Return true if no entries are inside of [`Repeater`](crate::Repeater)
    pub async fn is_empty(self) -> Result<bool, ConnectionLost> {
        let (reply_tx, reply_rx) = channel();

        self.tx.send(Message::IsEmtpy(reply_tx)).await?;
        Ok(reply_rx.await?)
    }
}

/// Error type for losing connecting to the [`Repeater`](crate::Repeater)
#[derive(Debug)]
pub struct ConnectionLost;

impl Display for ConnectionLost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Connection to repeater lost")
    }
}

impl<T> From<mpsc::error::SendError<T>> for ConnectionLost {
    fn from(_: mpsc::error::SendError<T>) -> Self {
        Self
    }
}

impl From<oneshot::error::RecvError> for ConnectionLost {
    fn from(_: oneshot::error::RecvError) -> Self {
        Self
    }
}
