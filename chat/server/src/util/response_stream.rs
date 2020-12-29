use futures::channel::oneshot;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use tokio::stream::Stream;

pub struct ResponseStream<T> {
    inner: Pin<Box<dyn Stream<Item = T> + Send + Sync>>,
    chan: Option<oneshot::Sender<bool>>,
}

impl<T> ResponseStream<T> {
    pub fn new<S>(stream: S) -> ResponseStream<T>
    where
        S: Stream<Item = T> + Send + Sync + 'static,
    {
        ResponseStream {
            inner: Box::pin(stream),
            chan: None,
        }
    }

    pub fn new_with_close_notification<S>(
        chan: oneshot::Sender<bool>,
        stream: S,
    ) -> ResponseStream<T>
    where
        S: Stream<Item = T> + Send + Sync + 'static,
    {
        ResponseStream {
            inner: Box::pin(stream),
            chan: Some(chan),
        }
    }
}

impl<T> Stream for ResponseStream<T> {
    type Item = T;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        self.inner.as_mut().as_mut().poll_next(cx)
    }
}

impl<T> Drop for ResponseStream<T> {
    fn drop(&mut self) {
        let chan = self.chan.take();
        if let Some(chan) = chan {
            chan.send(true).unwrap();
        }
    }
}
