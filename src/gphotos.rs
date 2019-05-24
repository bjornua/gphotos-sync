use crate::open;

use futures::{sync::oneshot::channel, Future};
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};
use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use tokio::runtime::current_thread;
#[derive(Debug)]
enum OpenAuthenticationURLError {
    NonZeroExitCode,
    IOError(io::Error),
}

fn open_authentication_url() -> Result<(), OpenAuthenticationURLError> {
    let status =
        open::that("http://127.0.0.1:3000").map_err(OpenAuthenticationURLError::IOError)?;
    match status.success() {
        true => Ok(()),
        false => Err(OpenAuthenticationURLError::NonZeroExitCode),
    }
}

fn hello_world(_req: Request<Body>) -> Response<Body> {
    Response::new(Body::from("Hello, World!"))
}

fn get_authentication_response() -> Result<(), OpenAuthenticationURLError> {
    let addr = ([127, 0, 0, 1], 3000).into();
    let (shutdown_sender, shutdown_receiver) = channel();
    let shutdown_sender = Rc::new(RefCell::new(Some(shutdown_sender)));
    let make_service = move || {
        let lol = Rc::clone(&shutdown_sender);
        service_fn_ok(move |r| {
            lol.borrow_mut().take().and_then(|r| r.send(()).ok());
            hello_world(r)
        })
    };
    let exec = current_thread::TaskExecutor::current();
    let server = Server::bind(&addr)
        .executor(exec)
        .serve(make_service)
        .with_graceful_shutdown(shutdown_receiver)
        .map_err(|e| eprintln!("server error: {}", e));

    let _ = current_thread::Runtime::new()
        .unwrap()
        .spawn(server)
        .run()
        .unwrap();
    Ok(())
}

#[derive(Debug)]
enum AuthenticationError {
    OpenAuthenticationURL(OpenAuthenticationURLError),
    AuthenticationServer,
}

fn authenticate() -> Result<(), AuthenticationError> {
    open_authentication_url().map_err(AuthenticationError::OpenAuthenticationURL)?;
    let _result = get_authentication_response();
    return Ok(());
}

pub fn main() {
    println!("{:?}", authenticate());
}
