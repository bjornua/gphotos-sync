use crate::lib::gauth;
use crate::lib::gphotos;
use crate::lib::hash::{hash_file, HashFileError, Hashes};

#[derive(Debug)]
pub enum UploadError {
    RefreshCredentials(gauth::RefreshCredentialsError),
    HashFile(std::path::PathBuf, HashFileError),
    UploadFile(gphotos::UploadFileError),
    BatchCreate(gphotos::BatchCreateError),
}

pub async fn upload_many<U: AsRef<std::path::Path>, T: Iterator<Item = U>>(
    credentials: &mut gauth::Credentials,
    uploaded_files: &mut Hashes,
    paths: T,
) -> Result<(), UploadError> {
    let mut upload_tokens: Vec<String> = Vec::with_capacity(50);
    let mut uploaded_hashes = Hashes::new();
    for path_u in paths {
        let path = path_u.as_ref();
        let (_hash_file_size, hash) =
            hash_file(&path).map_err(|e| UploadError::HashFile(path.into(), e))?;

        if uploaded_files.contains(&hash) {
            continue;
        }
        gauth::refresh_credentials_if_needed(credentials)
            .await
            .map_err(UploadError::RefreshCredentials)?;

        upload_tokens.push(
            gphotos::upload_file(&credentials.access_token, &path)
                .await
                .map_err(UploadError::UploadFile)?,
        );
        uploaded_hashes.insert(hash);

        if upload_tokens.len() >= 50 {
            gauth::refresh_credentials_if_needed(credentials)
                .await
                .map_err(UploadError::RefreshCredentials)?;

            gphotos::batch_create(&credentials.access_token, &upload_tokens)
                .await
                .map_err(UploadError::BatchCreate)?;

            upload_tokens.truncate(0);
            uploaded_files.append(&mut uploaded_hashes);
        }
    }
    gauth::refresh_credentials_if_needed(credentials)
        .await
        .map_err(UploadError::RefreshCredentials)?;

    gphotos::batch_create(&credentials.access_token, &upload_tokens)
        .await
        .map_err(UploadError::BatchCreate)?;

    uploaded_files.append(&mut uploaded_hashes);

    Ok(())
}
