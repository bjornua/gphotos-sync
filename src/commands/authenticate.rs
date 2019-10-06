use crate::open;
use clap::{App, ArgMatches, SubCommand};
use futures::Stream;
use futures::{sync::oneshot::channel, Future};
use hyper;
use hyper::service::service_fn_ok;
use percent_encoding;
use reqwest;
use serde;
use std::borrow::Cow;
use std::cell::RefCell;
use std::ffi::OsString;
use std::path;
use std::rc::Rc;
use std::thread::spawn;
use tokio::runtime::current_thread;

pub fn get_subcommand() -> App<'static, 'static> {
    SubCommand::with_name("authenticate").about("Authenticate with Google")
}

#[derive(serde::Serialize)]
struct Config {
    refresh_token: String,
}

pub fn main(_matches: &ArgMatches) {
    if path::Path::new("./config.json").exists() {
        println!("Configuration file already exists. Cannot overwrite.");
        return;
    }
    let refresh_token = match oauth() {
        Ok(refresh_token) => refresh_token,
        Err(error) => {
            println!("{:?}", error);
            return;
        }
    };
    let test = serde_json::to_string_pretty(&Config { refresh_token }).unwrap();
    println!("{}", test);
}

#[derive(Debug)]
enum OauthError {
    OauthAuthError(OauthAuthError),
    OauthTokenError(OauthTokenError),
}

fn oauth() -> Result<String, OauthError> {
    oauth_start_browser();
    let auth_code_future = oauth_auth().map_err(OauthError::OauthAuthError);

    let refresh_token_future =
        auth_code_future.and_then(|code| oauth_token(&code).map_err(OauthError::OauthTokenError));

    let response_code = current_thread::Runtime::new()
        .unwrap()
        .block_on(refresh_token_future)?;

    return Ok(response_code);
}

const CLIENT_ID: &'static str =
    "529339861110-lnubi506ma1cj16dtfl0sltdrepp0tmm.apps.googleusercontent.com";
const CLIENT_SECRET: &'static str = "pvykCj4vs1-JVVPazC-F8xht";
const ENDPOINT: &'static str = "https://accounts.google.com/o/oauth2/v2/auth";
const SCOPE: &'static str = "https://www.googleapis.com/auth/photoslibrary.appendonly";
const REDIRECT_URI: &'static str = "http://localhost:3000/";

fn oauth_start_browser() -> () {
    open_spawn(format!(
        "{:}?response_type=code&client_id={:}&redirect_uri={:}&scope={:}",
        ENDPOINT,
        percent_encoding::utf8_percent_encode(CLIENT_ID, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(REDIRECT_URI, percent_encoding::NON_ALPHANUMERIC),
        percent_encoding::utf8_percent_encode(SCOPE, percent_encoding::NON_ALPHANUMERIC),
    ));
}

fn parse_oauth_return(
    req: hyper::Request<hyper::Body>,
) -> (hyper::Response<hyper::Body>, Option<String>) {
    let code = req
        .uri()
        .query()
        .and_then(|q| get_query_parameter("code", q));
    (
        hyper::Response::new(hyper::Body::from(
            "sd-photo-uploader is now authenticated. You can close this page now.",
        )),
        code.map(|s| s.into_owned()),
    )
}

#[derive(Debug)]
enum OauthAuthError {
    AuthResponseServer(hyper::Error),
    ResultCanceled(futures::Canceled),
}

fn oauth_auth() -> impl Future<Item = String, Error = OauthAuthError> {
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

    let server = hyper::Server::bind(&addr)
        .executor(exec)
        .serve(make_service)
        .with_graceful_shutdown(shutdown_receiver)
        .map_err(OauthAuthError::AuthResponseServer);

    return server
        .join(result_receiver.map_err(OauthAuthError::ResultCanceled))
        .map(|(_, m)| m);
    ;
}

fn parse_query_string<'a>(query: &'a str) -> impl Iterator<Item = (Cow<'a, str>, Cow<'a, str>)> {
    query.split('&').map(|a| {
        let mut pair = a.splitn(2, '=');
        return (
            percent_encoding::percent_decode(pair.next().unwrap().as_ref())
                .decode_utf8()
                .unwrap(),
            pair.next()
                .map(|s| {
                    percent_encoding::percent_decode(s.as_ref())
                        .decode_utf8()
                        .unwrap()
                })
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
    spawn(|| open::that(url).ok());
}

#[derive(Debug)]
pub enum OauthTokenError {
    ReqwestError(reqwest::Error),
    ReadBodyError(reqwest::Error),
    UnhandledResponse {
        error: serde_json::Error,
        body: Vec<u8>,
    },
}
#[derive(serde::Deserialize, Debug)]
struct Response {
    access_token: String,
    expires_in: u64,
    refresh_token: String,
    scope: String,
    token_type: String,
}

// Do some parsing with serde here:
// https://github.com/serde-rs/serde

pub fn oauth_token(code: &str) -> impl Future<Item = String, Error = OauthTokenError> {
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
        .map_err(OauthTokenError::ReqwestError)
        .and_then(|response| {
            return response
                .into_body()
                .collect()
                .map_err(OauthTokenError::ReadBodyError);
        })
        .and_then(|body| {
            let body: Vec<_> = body.into_iter().flatten().collect();
            return serde_json::from_slice::<Response>(&body)
                .map(|x| x.refresh_token)
                .map_err(|error| OauthTokenError::UnhandledResponse { error, body });
        });
}
