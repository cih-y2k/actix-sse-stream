extern crate bytes;
extern crate tokio;
extern crate futures;
extern crate actix;
extern crate actix_web;

use std::time::{Duration, Instant};

use bytes::{Bytes};
use tokio::timer::Interval;
use futures::{Future, Async, Poll, Stream, sync::oneshot::*};
use actix::*;
use actix_web::{server, App, HttpRequest, HttpResponse, Error};

struct SSE {
    interval: Interval,
    tx: Sender<()>,
    rx: Receiver<()>
}

impl Stream for SSE {
    type Item = Bytes;
    type Error = Error;
    fn poll(&mut self) -> Poll<Option<Bytes>, Error> {
        match self.rx.poll() {
            Ok(Async::Ready(_)) => Ok(Async::Ready(Some(Bytes::from(&b"data: from chat server\n"[..])))),
            Ok(Async::NotReady) => {
                match self.interval.poll() {
                    Ok(Async::Ready(_)) => Ok(Async::Ready(Some(Bytes::from(&b"data: from interval\n"[..])))),
                    Ok(Async::NotReady) => Ok(Async::NotReady),
                    Err(_) => Ok(Async::Ready(None))
                }
            }
            Err(_) => Ok(Async::Ready(None))
        }
    }
}

fn handle_sse<'r>(_req: &'r HttpRequest) -> HttpResponse {
    let (tx, rx) = channel::<()>();
    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming( SSE { interval: Interval::new(Instant::now(), Duration::from_millis(5000)),
                        tx, rx
        })
}

fn main() {
    server::new(
        || App::new()
            .resource("/", |r| r.f(handle_sse)))
        .bind("127.0.0.1:8088").expect("Can not bind to 127.0.0.1:8088")
        .run();
}