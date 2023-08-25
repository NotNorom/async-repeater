use tokio::sync::{
    mpsc::{self},
    oneshot::{self, channel},
};

use crate::entry::RepeaterEntry;

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

    /// Insert entry into [`crate::Repeater`]
    pub async fn insert(&self, e: E) -> Result<(), ConnectionLost> {
        Ok(self.tx.send(Message::Insert(e)).await?)
    }

    pub async fn remove(&self, key: E::Key) -> Result<(), ConnectionLost> {
        Ok(self.tx.send(Message::Remove(key)).await?)
    }

    pub async fn clear(&self) -> Result<(), ConnectionLost> {
        Ok(self.tx.send(Message::Clear).await?)
    }

    pub async fn stop(self) -> Result<(), ConnectionLost> {
        Ok(self.tx.send(Message::Stop).await?)
    }

    pub async fn len(self) -> Result<usize, ConnectionLost> {
        let (reply_tx, reply_rx) = channel();

        self.tx.send(Message::Len(reply_tx)).await?;
        Ok(reply_rx.await?)
    }

    pub async fn is_empty(self) -> Result<bool, ConnectionLost> {
        let (reply_tx, reply_rx) = channel();

        self.tx.send(Message::IsEmtpy(reply_tx)).await?;
        Ok(reply_rx.await?)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Connection to repeater lost")]
pub struct ConnectionLost;

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
