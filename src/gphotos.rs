#[derive(Debug)]
pub enum UploadFileError {
    ReqwestError {
        access_token: String,
        inner: reqwest::Error,
    },
    GetAccessTokenError(crate::gauth::GetAccessTokenError),
    Duplicate {
        access_token: String,
    },
}

pub struct UploadFileOk {
    access_token: String,
    upload_token: String,
}

pub async fn upload_file(
    access_token: String,
    refresh_token: &str,
    path: &std::path::Path,
) -> Result<UploadFileOk, UploadFileError> {
    unimplemented!();
    // let response = reqwest::Client::new()
    //     .post("https://photoslibrary.googleapis.com/v1/uploads")
    //     .header("Authorization", "Bearer oauth2-token")
    //     .header("Content-type", "application/octet-stream")
    //     .header("X-Goog-Upload-File-Name", "filename")
    //     .header("X-Goog-Upload-Protocol", "raw")
    //     .form(&[
    //         ("code", code),
    //         ("client_id", CLIENT_ID),
    //         ("client_secret", CLIENT_SECRET),
    //         ("redirect_uri", REDIRECT_URI),
    //         ("grant_type", "authorization_code"),
    //     ])
    //     .send()
    //     .await
    //     .map_err(UploadError::ReqwestError);

    // println!("Uploading: {:?}", path);

    // return Ok(UploadOk {
    //     access_token,
    //     upload_token: String::new(),
    // });
}
