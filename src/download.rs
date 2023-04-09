use std::{cmp::min, fs::File, io::Write};

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

pub struct Downloader {
    client: Client,

    output: String,
}

impl Downloader {
    pub fn new(output: &str) -> Downloader {
        let client = Client::new();

        Downloader {
            client,
            output: output.to_string(),
        }
    }

    pub async fn download(&self, url: &str) -> Result<(), String> {
        // check output directory exist or mkdir
        if !std::path::Path::new(&self.output).exists() {
            std::fs::create_dir_all(&self.output).or(Err(format!(
                "Failed to create directory '{}'",
                &self.output
            )))?;
        }

        // get file name from url
        let path = &format!(
            "{}/{}",
            self.output,
            url.split('/').last().unwrap_or("unknown")
        );

        // Reqwest setup
        let res = self
            .client
            .get(url)
            .send()
            .await
            .or(Err(format!("Failed to GET from '{}'", &url)))?;
        let total_size = res
            .content_length()
            .ok_or(format!("Failed to get content length from '{}'", &url))?;

        // Indicatif setup
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .map_err(|e| format!("Failed to set progress bar style: {}", e))?
            .progress_chars("##-"));
        pb.set_message(format!("Downloading {}", url));

        // download chunks
        let mut file = File::create(path).or(Err(format!("Failed to create file '{}'", path)))?;
        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item.or(Err(format!("Error while downloading file")))?;
            file.write_all(&chunk)
                .or(Err(format!("Error while writing to file")))?;
            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            pb.set_position(new);
        }

        pb.finish_with_message(format!("Downloaded {} to {}", url, path));
        return Ok(());
    }
}
