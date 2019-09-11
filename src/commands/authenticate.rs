use crate::open;
use clap::{App, ArgMatches, SubCommand};
use std::ffi::OsString;
use std::thread::spawn;

use futures::{sync::oneshot::channel, Future};
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};
use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use tokio::runtime::current_thread;
use url::percent_encoding::percent_decode;

pub fn get_subcommand() -> App<'static, 'static> {
    SubCommand::with_name("authenticate").about("Authenticate with Google")
}

pub fn main(_matches: &ArgMatches) {
    println!("{:?}", authenticate());
}

#[derive(Debug)]
enum AuthenticationError {
    GetAuthenticationResponseError(GetAuthenticationResponseError),
    RedeemResponseCodeError(RedeemResponseCodeError),
}

fn authenticate() -> Result<String, AuthenticationError> {
    open_authentication_url();
    let result = get_authentication_response()
        .map_err(AuthenticationError::GetAuthenticationResponseError)
        .and_then(|code| {
            redeem_response_code(&code).map_err(AuthenticationError::RedeemResponseCodeError)
        });

    let response_code = current_thread::Runtime::new().unwrap().block_on(result)?;

    return Ok(response_code);
}

const CLIENT_ID: &'static str =
    "529339861110-lnubi506ma1cj16dtfl0sltdrepp0tmm.apps.googleusercontent.com";
const ENDPOINT: &'static str = "https://accounts.google.com/o/oauth2/v2/auth";
const SCOPE: &'static str = "https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fphotoslibrary.appendonly";
const REDIRECT_URL: &'static str = "http%3A%2F%2Flocalhost%3A3000%2F";

fn open_authentication_url() -> () {
    open_spawn(format!(
        "{:}?response_type=code&client_id={:}&redirect_uri={:}&scope={:}",
        ENDPOINT, CLIENT_ID, REDIRECT_URL, SCOPE
    ));
}

fn parse_oauth_return(req: Request<Body>) -> (Response<Body>, Option<String>) {
    let code = req
        .uri()
        .query()
        .and_then(|q| get_query_parameter("code", q));
    (
        Response::new(Body::from(
            "sd-photo-uploader is now authenticated. You can close this page now.",
        )),
        code.map(|s| s.into_owned()),
    )
}

#[derive(Debug)]
enum GetAuthenticationResponseError {
    AuthResponseServer(hyper::Error),
    ResultCanceled(futures::Canceled),
}

fn get_authentication_response(
) -> impl Future<Item = String, Error = GetAuthenticationResponseError> {
    let addr = ([127, 0, 0, 1], 3000).into();
    let (result_sender, result_receiver) = channel();
    let result_sender = Arc::new(Mutex::new(Some(result_sender)));
    let make_service = move || {
        let result_sender = Arc::clone(&result_sender);
        service_fn_ok(move |r| {
            let (response, code) = parse_oauth_return(r);
            if let Some(code) = code {
                result_sender
                    .lock()
                    .unwrap()
                    .take()
                    .and_then(|r| r.send(code).ok());
            };
            response
        })
    };

    let (shutdown_sender, shutdown_receiver) = channel();
    let result_receiver = result_receiver.map(|code| {
        let _ = shutdown_sender.send(());
        code
    });
;
    let server = Server::bind(&addr)
        .serve(make_service)
        .with_graceful_shutdown(shutdown_receiver)
        .map_err(GetAuthenticationResponseError::AuthResponseServer);

    return server
        .join(result_receiver.map_err(GetAuthenticationResponseError::ResultCanceled))
        .map(|(_, m)| m);
    ;
}

fn parse_query_string<'a>(query: &'a str) -> impl Iterator<Item = (Cow<'a, str>, Cow<'a, str>)> {
    query.split('&').map(|a| {
        let mut lol = a.splitn(2, '=');
        return (
            percent_decode(lol.next().unwrap().as_ref())
                .decode_utf8()
                .unwrap(),
            lol.next()
                .map(|s| percent_decode(s.as_ref()).decode_utf8().unwrap())
                .unwrap_or(Cow::Borrowed("")),
        );
    })
}

fn get_query_parameter<'a, 'b>(search_key: &'a str, query: &'b str) -> Option<Cow<'b, str>> {
    parse_query_string(query)
        .find(|(key, _)| key == search_key)
        .map(|(_, value)| value)
}

pub fn open_spawn<T: Into<OsString>>(url: T) {
    let url = url.into();
    spawn(move || {
        open::that(url).ok();
    });
}

#[derive(Debug)]
enum RedeemResponseCodeError {}
pub fn redeem_response_code(
    code: &str,
) -> impl Future<Item = String, Error = RedeemResponseCodeError> {

}
