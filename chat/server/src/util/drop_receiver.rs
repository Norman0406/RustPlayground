use core::task::{Context, Poll};
use futures::channel::oneshot;
use std::ops::Deref;
use tokio::sync::mpsc;

pub struct DropReceiver<T> {
    pub inner: mpsc::Receiver<T>,
    chan: Option<oneshot::Sender<bool>>,
}

impl<T> DropReceiver<T> {
    pub fn new(receiver: mpsc::Receiver<T>, chan: oneshot::Sender<bool>) -> DropReceiver<T> {
        DropReceiver {
            inner: receiver,
            chan: Some(chan),
        }
    }
}

impl<T> tokio::stream::Stream for DropReceiver<T> {
    type Item = T;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        self.inner.poll_recv(cx)
    }
}

impl<T> Deref for DropReceiver<T> {
    type Target = mpsc::Receiver<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Drop for DropReceiver<T> {
    fn drop(&mut self) {
        let chan = self.chan.take();
        if let Some(chan) = chan {
            chan.send(true).unwrap();
        }
    }
}
