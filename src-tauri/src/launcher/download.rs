use futures_util::StreamExt;
use reqwest::Client;
use serde::{self, Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::{
    error::Error,
    fs::{self, remove_file, rename, File},
    io::Write,
    path::PathBuf,
};

#[derive(Serialize, Deserialize)]
pub struct DownloadObject {
    pub url: String,
    pub size: Option<u64>,    // expected size
    pub sha1: Option<String>, // expected hash
    pub file_path: PathBuf,
}

impl DownloadObject {
    // tried to implement this with a .part file so if the users pc crashes or smth the download can continue, but i found out mojangs servers dont support http range :/
    pub async fn download_file(&self, mut on_chunk: impl FnMut(u64)) -> Result<(), Box<dyn Error>> {
        let minecraft_dir = dirs::home_dir()
            .ok_or("home dir dont exists")?
            .join(".minecraft");
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

        // 26.2.jar.part
        let part = file_path.with_extension(format!(
            "{}.part",
            file_path
                .extension()
                .and_then(|extension| extension.to_str())
                .unwrap_or("")
        ));

        let client = Client::new();

        let res = client.get(&self.url).send().await?.error_for_status()?;

        let mut file = File::create(&part).or(Err(format!(
            "failed to create file, path: {}",
            &part.display()
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
            on_chunk(chunk.len() as u64);
        }

        file.flush()?;

        if let Some(size) = self.size {
            if downloaded_bytes != size {
                remove_file(&part)?;

                return Err(format!("size mismatch"))?;
            }
        }

        if let Some(sha1) = &self.sha1 {
            let last_sha1 = hex::encode(hasher.finalize());
            if &last_sha1 != sha1 {
                remove_file(&part)?;

                return Err(format!("sha1 mismatch"))?;
            }
        }

        rename(&part, file_path)?;
        Ok(())
    }
}
