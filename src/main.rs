extern crate hyper;

#[macro_use]
extern crate log;
extern crate simple_logger;

extern crate tokio;
extern crate tokio_current_thread;
extern crate tokio_executor;
extern crate tokio_io;
extern crate tokio_reactor;
extern crate tokio_timer;

use std::{error, fmt};
use std::net::SocketAddr;

use hyper::{Body, Response};
use hyper::server::conn::Http;
use hyper::service;

use tokio::prelude::*;

use tokio_current_thread::{self as current_thread, CurrentThread, Entered};
use tokio_executor::park::Park;
use tokio_reactor::{Handle, Reactor};
use tokio_timer::{timer, Timer};

#[derive(Debug)]
pub struct CantFail;

impl error::Error for CantFail {
    fn description(&self) -> &str { "" }
    fn cause(&self) -> Option<&error::Error> { None }
}

impl fmt::Display for CantFail {
    fn fmt(&self, _f: &mut fmt::Formatter) -> Result<(), fmt::Error> { Ok(()) }
}

fn create_serve_all<S>(handle: &Handle, addr: SocketAddr, new_service: S) -> impl Future<Item = (), Error = ()>
where S: service::NewService<ReqBody=hyper::Body, ResBody=hyper::Body>,
      S::Error: Into<Box<::std::error::Error + Send + Sync>>,
      <S as service::NewService>::Future: 'static,
      <S as service::NewService>::Service: service::Service,
      <S as service::NewService>::InitError: std::fmt::Debug,
      <S::Service as service::Service>::Future: 'static + Send,
{
    let serve = Http::new().serve_addr_handle(&addr, handle, new_service).unwrap();

    let serve_all = serve.for_each(move |conn| {
        trace!("spawning connection future onto the current thread");
        current_thread::spawn(conn.map(|_| ()).map_err(|e| { error!("serve.for_each: {:?}", e); () }));
        Ok(())
    }).map_err(|e| { error!("serve_all: {:?}", e); () });

    serve_all
}

pub fn serve_http<'a, P>(handle: &Handle, executor: &mut Entered<'a, P>, addr: SocketAddr)
where P: Park,
{
    let serve_all = create_serve_all(handle, addr, || {
        service::service_fn(|_req| {
            future::ok::<Response<Body>, CantFail>(Response::new(Body::from("HELLO WORLD!")))
        })
    });
    trace!("spawning server future onto the current thread");
    executor.spawn(Box::new(serve_all));
}

fn main() -> Result<(), ()> {
    simple_logger::init().unwrap();

    let reactor = Reactor::new()
        .map_err(|e| error!("creating reactor failed: {}", e))?;
    let reactor_handle = reactor.handle();
    let timer = Timer::new(reactor);
    let timer_handle = timer.handle();
    let mut executor = CurrentThread::new_with_park(timer);

    let mut enter = tokio_executor::enter()
        .expect("Multiple executors at once");

    tokio_reactor::with_default(
        &reactor_handle,
        &mut enter,
        |enter| timer::with_default(
            &timer_handle,
            enter,
            |enter| {
                let mut default_executor =
                    current_thread::TaskExecutor::current();
                tokio_executor::with_default(
                    &mut default_executor,
                    enter,
                    |enter|{
                        let mut entered = executor.enter(enter);
                        let addr = "127.0.0.1:4000".parse().unwrap();
                        serve_http(&reactor_handle, &mut entered, addr);
                        entered.run().expect("exited main executor");
                    }
                )
            }
        )
    );

    Ok(())
}
