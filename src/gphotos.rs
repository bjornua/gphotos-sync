use hyper;
use open;

use futures::channel::oneshot;
use percent_encoding;
use reqwest;
use serde;
use std::borrow::Cow;
use std::ffi::OsString;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
#[derive(Debug)]
pub enum OauthError {
    OauthAuthError(OauthAuthError),
    OauthTokenError(OauthTokenError),
}

pub async fn oauth() -> Result<String, OauthError> {
    oauth_start_browser();
    let auth_code = oauth_auth().await.map_err(OauthError::OauthAuthError)?;
    let refresh_token = oauth_token(&auth_code)
        .await
        .map_err(OauthError::OauthTokenError)?;
    return Ok(refresh_token);
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
pub enum OauthAuthError {
    AuthResponseServer(hyper::Error),
}

async fn oauth_auth() -> Result<String, OauthAuthError> {
    let addr = ([127, 0, 0, 1], 3000).into();
    let (result_sender, result_receiver) = oneshot::channel::<String>();
    let result_sender = Arc::new(Mutex::new(Some(result_sender)));
    let make_service = hyper::service::make_service_fn(move |_| {
        let result_sender = Arc::clone(&result_sender);
        async {
            return Ok::<_, hyper::Error>(hyper::service::service_fn(move |r| {
                let result_sender = Arc::clone(&result_sender);
                async move {
                    let (response, code) = parse_oauth_return(r);
                    if let Some(code) = code {
                        result_sender
                            .lock()
                            .unwrap()
                            .take()
                            .and_then(|r| r.send(code).ok());
                    };
                    return Ok::<_, hyper::Error>(response);
                }
            }));
        }
    });

    let (shutdown_sender, shutdown_receiver) = oneshot::channel::<String>();

    let server = hyper::Server::bind(&addr)
        .serve(make_service)
        .with_graceful_shutdown(async {
            let code = result_receiver.await.unwrap();
            shutdown_sender.send(code).unwrap();
        });

    server.await.map_err(OauthAuthError::AuthResponseServer)?;
    return Ok(shutdown_receiver.await.unwrap());
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
}
#[derive(serde::Deserialize)]
struct Response {
    refresh_token: String,
}

pub async fn oauth_token(code: &str) -> Result<String, OauthTokenError> {
    let response = reqwest::Client::new()
        .post("https://www.googleapis.com/oauth2/v4/token")
        .form(&[
            ("code", code),
            ("client_id", CLIENT_ID),
            ("client_secret", CLIENT_SECRET),
            ("redirect_uri", REDIRECT_URI),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(OauthTokenError::ReqwestError)?;

    let payload = response
        .json::<Response>()
        .await
        .map_err(OauthTokenError::ReadBodyError)?;
    return Ok(payload.refresh_token);
}

#[derive(Debug)]
pub enum UploadError {
    ReqwestError {
        access_token: String,
        inner: reqwest::Error,
    },
    Unauthorized,
    Duplicate {
        access_token: String,
    },
}

pub enum GetAccessTokenError {
    ReqwestError(reqwest::Error),
}

pub async fn get_access_token(refresh_token: String) -> Result<String, GetAccessTokenError> {
    unimplemented!()
}

pub struct UploadOk {access_token: String};
pub async fn upload_file(
    access_token: String,
    refresh_token: &str,
    path: &std::path::Path,
) -> Result<UploadOk, UploadError> {
    let response = reqwest::Client::new()
        .post("https://photoslibrary.googleapis.com/v1/uploads")
        .header("Authorization", "Bearer oauth2-token")
        .header("Content-type", "application/octet-stream")
        .header("X-Goog-Upload-File-Name", "filename")
        .header("X-Goog-Upload-Protocol", "raw")
        .form(&[
            ("code", code),
            ("client_id", CLIENT_ID),
            ("client_secret", CLIENT_SECRET),
            ("redirect_uri", REDIRECT_URI),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(UploadError::ReqwestError);

    println!("Uploading: {:?}", path);

    return Ok(UploadOk {
        access_token,
        upload_token: String::new(),
    });
}
