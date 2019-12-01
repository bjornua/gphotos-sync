use std::io::Read;

#[derive(Debug)]
pub enum UploadFileError {
    Request(reqwest::Error),
    ReadFile(std::io::Error),
    ResponseReadBody(reqwest::Error),
}

pub async fn upload_file(
    access_token: &str,
    path: &std::path::Path,
) -> Result<String, UploadFileError> {
    let mut file = std::fs::File::open(path).unwrap();
    let mut buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(UploadFileError::ReadFile)?;

    println!("Uploading {:?}", path);
    return reqwest::Client::new()
        .post("https://photoslibrary.googleapis.com/v1/uploads")
        .bearer_auth(access_token)
        .header("Content-type", "application/octet-stream")
        .header("X-Goog-Upload-File-Name", "filename")
        .header("X-Goog-Upload-Protocol", "raw")
        .body(buffer)
        .send()
        .await
        .map_err(UploadFileError::Request)?
        .text()
        .await
        .map_err(UploadFileError::ResponseReadBody);
}

#[derive(Debug)]
pub enum BatchCreateError {
    Request(reqwest::Error),
    UnhandledResponse(reqwest::Error),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SimpleMediaItem {
    upload_token: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MediaItem {
    description: String,
    simple_media_item: SimpleMediaItem,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchCreate {
    new_media_items: Vec<MediaItem>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchCreateResponse {
    new_media_item_results: Vec<MediaItemResult>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct MediaItemResult {
    upload_token: String,
    status: MediaItemResultStatus,
}

#[derive(serde::Deserialize, Debug)]
pub struct MediaItemResultStatus {
    code: Option<u64>,
    message: String,
}

pub async fn batch_create(
    access_token: &str,
    tokens: &[String],
) -> Result<Vec<(String, Result<(), MediaItemResultStatus>)>, BatchCreateError> {
    if tokens.len() == 0 {
        return Ok(Vec::new());
    }

    let media_items = BatchCreate {
        new_media_items: tokens
            .into_iter()
            .map(|token| -> MediaItem {
                MediaItem {
                    description: String::new(),
                    simple_media_item: SimpleMediaItem {
                        upload_token: token.clone(),
                    },
                }
            })
            .collect(),
    };
    let response = reqwest::Client::new()
        .post("https://photoslibrary.googleapis.com/v1/mediaItems:batchCreate")
        .bearer_auth(access_token)
        .header("Content-type", "application/octet-stream")
        .header("X-Goog-Upload-File-Name", "filename")
        .header("X-Goog-Upload-Protocol", "raw")
        .json(&media_items)
        .send()
        .await
        .map_err(BatchCreateError::Request)?
        .json::<BatchCreateResponse>()
        .await
        .map_err(BatchCreateError::UnhandledResponse)?;
    let test: Vec<_> = response
        .new_media_item_results
        .into_iter()
        .map(|item| {
            (
                item.upload_token,
                match item.status.code {
                    Some(_) => Err(item.status),
                    None => Ok(()),
                },
            )
        })
        .collect();
    return Ok(test);
}
