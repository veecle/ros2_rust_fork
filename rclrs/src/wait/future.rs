use super::{ReadyEntities, WaitSet};
use crate::Node;
use futures::channel::mpsc::{self, Receiver};
use futures::Stream;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    thread::JoinHandle,
};

pub enum WaitSetFuture<'node> {
    Fresh(&'node Node),
    Spinning {
        thread: JoinHandle<()>,
        rx: Receiver<ReadyEntities>,
    },
}

impl<'node> WaitSetFuture<'node> {
    pub fn new(node: &'node Node) -> Self {
        Self::Fresh(node)
    }
}

// TODO: Check that this is safe in detail
unsafe impl Send for WaitSet {}

impl<'node> Future for WaitSetFuture<'node> {
    type Output = ReadyEntities;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let next = match *self {
            WaitSetFuture::Fresh(node) => {
                let mut wait_set = WaitSet::new_for_node(node).unwrap();
                let (mut tx, rx) = mpsc::channel(10);
                let thread = std::thread::spawn(move || loop {
                    let ready_entities = wait_set.wait(None).unwrap();
                    tx.try_send(ready_entities).unwrap();
                });
                Some(WaitSetFuture::Spinning { thread, rx })
            }
            WaitSetFuture::Spinning {
                ref thread,
                ref mut rx,
            } => {
                if let Poll::Ready(Some(ready_entities)) = Pin::new(rx).poll_next(cx) {
                    todo!()
                }
                None
            }
        };
        if let Some(val) = next {
            *self = val;
        }
        Poll::Pending
    }
}
