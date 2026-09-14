use futures_util::StreamExt;
use reqwest::Client;
use serde::{self, Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{
    error::Error,
    fs::{self, remove_file, rename, File},
    io::Write,
    path::PathBuf,
    time::Duration,
};
use tokio::time::sleep;

use crate::launcher::minecraft_dir::get_minecraft_dir;

const MAX_ATTEMPTS: u32 = 5;
const RETRY_BASE_DELAY: Duration = Duration::from_millis(500);
const RETRY_MAX_DELAY: Duration = Duration::from_secs(8);

#[derive(Serialize, Deserialize)]
pub struct DownloadObject {
    pub url: String,
    pub size: Option<u64>,    // expected size
    pub sha1: Option<String>, // expected hash
    pub file_path: PathBuf,
}

fn is_retryable(err: &(dyn Error + 'static)) -> bool {
    match err.downcast_ref::<reqwest::Error>() {
        Some(reqwest_err) => match reqwest_err.status() {
            None => !reqwest_err.is_builder(),
            Some(status) => status.is_server_error() || status.as_u16() == 429,
        },
        None => true,
    }
}

impl DownloadObject {
    pub async fn download_file(&self, mut on_chunk: impl FnMut(u64)) -> Result<(), Box<dyn Error>> {
        let minecraft_dir = get_minecraft_dir()?;
        let file_path = minecraft_dir.join(&self.file_path);
        if file_path.exists() {
            if let Some(size) = self.size {
                let metadata = fs::metadata(&file_path)?;

                if size == metadata.len() {
                    if let Some(sha1) = &self.sha1 {
                        let bytes = fs::read(&file_path)?;
                        let mut hasher = Sha1::new();

                        hasher.update(&bytes);
                        let hash = hasher.finalize();
                        let hash = hex::encode(hash);
                        if sha1 == &hash {
                            on_chunk(size);
                            return Ok(());
                        }
                    } else {
                        on_chunk(size);
                        return Ok(());
                    }
                }
            }
        }

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let part = file_path.with_extension(format!(
            "{}.part",
            file_path
                .extension()
                .and_then(|extension| extension.to_str())
                .unwrap_or("")
        ));

        let client = Client::new();

        let mut reported_bytes: u64 = 0;
        let mut attempt: u32 = 1;

        loop {
            let delay = match self
                .try_download_once(&client, &part, &mut reported_bytes, &mut on_chunk)
                .await
            {
                Ok(()) => {
                    rename(&part, &file_path)?;
                    return Ok(());
                }
                Err(err) => {
                    let _ = remove_file(&part);

                    if attempt >= MAX_ATTEMPTS || !is_retryable(err.as_ref()) {
                        return Err(err);
                    }

                    let delay = RETRY_BASE_DELAY
                        .saturating_mul(2u32.saturating_pow(attempt - 1))
                        .min(RETRY_MAX_DELAY);

                    eprintln!(
                        "[download] attempt {attempt}/{MAX_ATTEMPTS} failed for {}: {err} (retrying in {delay:?})",
                        self.url
                    );

                    delay
                }
            };

            sleep(delay).await;
            attempt += 1;
        }
    }

    async fn try_download_once(
        &self,
        client: &Client,
        part: &PathBuf,
        reported_bytes: &mut u64,
        on_chunk: &mut impl FnMut(u64),
    ) -> Result<(), Box<dyn Error>> {
        let res = client.get(&self.url).send().await?.error_for_status()?;

        let mut file = File::create(part).or(Err(format!(
            "failed to create file, path: {}",
            part.display()
        )))?;

        let mut stream = res.bytes_stream();
        let mut hasher = Sha1::new();
        let mut downloaded_bytes: u64 = 0;

        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk)
                .or(Err(format!("failed to write in file")))?;

            hasher.update(&chunk);
            downloaded_bytes += chunk.len() as u64;

            if downloaded_bytes > *reported_bytes {
                on_chunk(downloaded_bytes - *reported_bytes);
                *reported_bytes = downloaded_bytes;
            }
        }

        file.flush()?;

        if let Some(size) = self.size {
            if downloaded_bytes != size {
                return Err(format!("size mismatch"))?;
            }
        }

        if let Some(sha1) = &self.sha1 {
            let last_sha1 = hex::encode(hasher.finalize());
            if &last_sha1 != sha1 {
                return Err(format!("sha1 mismatch"))?;
            }
        }

        Ok(())
    }
}
