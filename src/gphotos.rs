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

fn get_authentication_response(port: u16) -> Result<(), OpenAuthenticationURLError> {
    let addr = ([127, 0, 0, 1], port).into();
    let (shutdown_sender, shutdown_receiver) = channel::<u64>();
    let make_service = move || {
        service_fn_ok(move |r| {
            (&shutdown_sender).send(64).ok();
            hello_world(r)
        })
    };
    let exec = current_thread::TaskExecutor::current();
    let server = Server::bind(&addr)
        .executor(exec)
        .serve(make_service)
        .with_graceful_shutdown(shutdown_receiver)
        .map_err(|e| eprintln!("server error: {}", e));
    println!("Running server");

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
    open_authentication_url(4000).map_err(AuthenticationError::OpenAuthenticationURL)?;
    let _result = get_authentication_response(4000);
    return Ok(());
}

pub fn main() {
    println!("{:?}", authenticate());
}
