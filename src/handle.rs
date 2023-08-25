use tokio::sync::mpsc::Sender;

use crate::entry::RepeaterEntry;

pub(crate) enum Message<E>
where
    E: RepeaterEntry + Clone + Unpin + 'static,
{
    Insert(E),
    Remove(E::Key),
    Clear,
    Stop,
}

#[derive(Debug, Clone)]
pub struct RepeaterHandle<E>
where
    E: RepeaterEntry + Clone + Unpin + 'static,
{
    tx: Sender<Message<E>>,
}

impl<E> RepeaterHandle<E>
where
    E: RepeaterEntry + Clone + Unpin + 'static,
{
    pub(crate) fn new(sender: Sender<Message<E>>) -> Self {
        Self { tx: sender }
    }

    pub async fn insert(&self, e: E) {
        let Err(err) = self.tx.send(Message::Insert(e)).await else { return };
        eprintln!("Error sending message: {err:?}");
    }

    pub async fn remove(&self, key: E::Key) {
        let Err(err) = self.tx.send(Message::Remove(key)).await else { return };
        eprintln!("Error sending message: {err:?}");
    }

    pub async fn clear(&self) {
        let Err(err) = self.tx.send(Message::Clear).await else { return };
        eprintln!("Error sending message: {err:?}");
    }

    pub async fn stop(self) {
        let Err(err) = self.tx.send(Message::Stop).await else { return };
        eprintln!("Error sending message: {err:?}");
    }
}
