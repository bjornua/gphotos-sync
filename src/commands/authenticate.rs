use crate::open;
use clap::{App, ArgMatches, SubCommand};
use futures::Stream;
use futures::{future, sync::oneshot::channel, Future};
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};
use percent_encoding::percent_decode;
use reqwest;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;
use std::ffi::OsString;
use std::io::Read;
use std::rc::Rc;
use std::string;
use std::thread::spawn;
use tokio::runtime::current_thread;
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
    // let result = get_authentication_response()
    //     .map_err(AuthenticationError::GetAuthenticationResponseError)
    //     .and_then(|code| {
    //         println!("{}", code);
    //         redeem_response_code(&code).map_err(AuthenticationError::RedeemResponseCodeError)
    //     });

    // let response_code = current_thread::Runtime::new().unwrap().block_on(result)?;
    let response_code = current_thread::Runtime::new().unwrap().block_on(redeem_response_code("").map_err(AuthenticationError::RedeemResponseCodeError))?;

    return Ok(response_code);
}

const CLIENT_ID: &'static str =
    "529339861110-lnubi506ma1cj16dtfl0sltdrepp0tmm.apps.googleusercontent.com";
const CLIENT_SECRET: &'static str = "pvykCj4vs1-JVVPazC-F8xht";
const ENDPOINT: &'static str = "https://accounts.google.com/o/oauth2/v2/auth";
const SCOPE: &'static str = "https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fphotoslibrary.appendonly";
const REDIRECT_URI: &'static str = "http%3A%2F%2Flocalhost%3A3000%2F";

fn open_authentication_url() -> () {
    open_spawn(format!(
        "{:}?response_type=code&client_id={:}&redirect_uri={:}&scope={:}",
        ENDPOINT, CLIENT_ID, REDIRECT_URI, SCOPE
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
    let result_sender = Rc::new(RefCell::new(Some(result_sender)));
    let make_service = move || {
        let result_sender = Rc::clone(&result_sender);
        service_fn_ok(move |r| {
            let (response, code) = parse_oauth_return(r);
            if let Some(code) = code {
                result_sender
                    .borrow_mut()
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

    let exec = current_thread::TaskExecutor::current();

    let server = Server::bind(&addr)
        .executor(exec)
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
pub enum RedeemResponseCodeError {
    ReqwestError(reqwest::Error),
    ResponseNotUTF8(string::FromUtf8Error),
    ReadBodyError(reqwest::Error),
}

// Do some parsing with serde here:
// https://github.com/serde-rs/serde

pub fn redeem_response_code(
    code: &str,
) -> impl Future<Item = String, Error = RedeemResponseCodeError> {
    return reqwest::r#async::Client::new()
        .post("https://www.googleapis.com/oauth2/v4/token")
        .form(&[
            ("code", code),
            ("client_id", CLIENT_ID),
            ("client_secret", CLIENT_SECRET),
            ("redirect_uri", REDIRECT_URI),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .map_err(RedeemResponseCodeError::ReqwestError)
        .and_then(|response| {
            println!("Status:\n{:?}\n\n", response.status());
            println!("Headers:\n{:?}\n\n", response.headers());

            return response
                .into_body()
                .collect()
                .map_err(RedeemResponseCodeError::ReadBodyError);
        })
        .and_then(|body| {
            let body = String::from_utf8(body.into_iter().flatten().collect())
                .map_err(RedeemResponseCodeError::ResponseNotUTF8)?;

            println!("Body:\n{}\n\n", body);
            return Ok(String::new());
        });
}

// POST /oauth2/v4/token HTTP/1.1
// Host: www.googleapis.com
// Content-Type: application/x-www-form-urlencoded

// code=4/P7q7W91a-oMsCeLvIaQm6bTrgtp7&
// client_id=your_client_id&
// client_secret=your_client_secret&
// redirect_uri=https://oauth2.example.com/code&
// grant_type=authorization_code
