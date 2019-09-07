use crate::open;

use futures::{sync::oneshot::channel, Canceled, Future};
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

fn open_authentication_url(port: u16) -> Result<(), OpenAuthenticationURLError> {
    let status = open::that(format!("http://127.0.0.1:{}", port))
        .map_err(OpenAuthenticationURLError::IOError)?;
    match status.success() {
        true => Ok(()),
        false => Err(OpenAuthenticationURLError::NonZeroExitCode),
    }
}

fn hello_world(_req: Request<Body>) -> Response<Body> {
    Response::new(Body::from("Hello, World!"))
}

#[derive(Debug)]
enum AuthenticationServerError {
    RuntimeCancelled(Canceled),
}

fn get_authentication_response(port: u16) -> Result<u64, AuthenticationServerError> {
    let addr = ([127, 0, 0, 1], port).into();
    let (result_sender, result_receiver) = channel::<u64>();
    let result_sender = Rc::new(RefCell::new(Some(result_sender)));
    let make_service = move || {
        let result_sender = Rc::clone(&result_sender);
        service_fn_ok(move |r| {
            result_sender
                .borrow_mut()
                .take()
                .and_then(|r| r.send(64).ok());
            hello_world(r)
        })
    };

    let (shutdown_sender, shutdown_receiver) = channel::<()>();
    let result_receiver = result_receiver.map(|x| {
        let _ = shutdown_sender.send(());
        return x;
    });

    let exec = current_thread::TaskExecutor::current();
    let server = Server::bind(&addr)
        .executor(exec)
        .serve(make_service)
        .with_graceful_shutdown(shutdown_receiver)
        .map_err(|e| eprintln!("server error: {}", e));
    println!("Running server");

    return current_thread::Runtime::new()
        .unwrap()
        .spawn(server)
        .block_on(result_receiver)
        .map_err(AuthenticationServerError::RuntimeCancelled);
}

#[derive(Debug)]
enum AuthenticationError {
    OpenAuthenticationURLError(OpenAuthenticationURLError),
    AuthenticationServerError(AuthenticationServerError),
}

fn authenticate() -> Result<u64, AuthenticationError> {
    open_authentication_url(4000).map_err(AuthenticationError::OpenAuthenticationURLError)?;
    let result = get_authentication_response(4000)
        .map_err(AuthenticationError::AuthenticationServerError)?;
    return Ok(result);
}

pub fn main() {
    println!("{:?}", authenticate());
}
