use crate::open;
use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};
use std::io;

#[derive(Debug)]
enum OpenAuthenticationURLError {
    NonZeroExitCode,
    IOError(io::Error),
}

fn open_authentication_url() -> Result<(), OpenAuthenticationURLError> {
    let status = open::that("https://google.com").map_err(OpenAuthenticationURLError::IOError)?;
    match status.success() {
        true => Ok(()),
        false => Err(OpenAuthenticationURLError::NonZeroExitCode),
    }
}

const PHRASE: &str = "Hello, World!";

fn hello_world(_req: Request<Body>) -> Response<Body> {
    Response::new(Body::from(PHRASE))
}

fn get_authentication_response() -> Result<(), OpenAuthenticationURLError> {
    // This is our socket address...
    let addr = ([127, 0, 0, 1], 3000).into();

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let new_svc = || {
        // service_fn_ok converts our function into a `Service`
        service_fn_ok(hello_world)
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));

    // Run this server for... forever!
    hyper::rt::run(server);
}

#[derive(Debug)]
enum AuthenticationError {
    OpenAuthenticationURL(OpenAuthenticationURLError),
    AuthenticationServer,
}

fn authenticate() -> Result<(), AuthenticationError> {
    open_authentication_url().map_err(AuthenticationError::OpenAuthenticationURL)?;
    return Ok(());
    // let result = start_response_server();
}

pub fn main() {
    println!("{:?}", authenticate());
}
