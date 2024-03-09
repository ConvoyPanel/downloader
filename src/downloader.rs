use std::env;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;

#[derive(Serialize, Deserialize, Debug)]
pub struct Template {
    pub name: String,
    pub vmid: i32,
    pub link: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Group {
    pub name: String,
    pub templates: Vec<Template>,
}

pub async fn fetch_templates() -> anyhow::Result<Vec<Group>> {
    let args: Vec<String> = env::args().collect();

    let default_url = "https://images.cdn.convoypanel.com/images.json".to_string();

    let url = args.get(1).unwrap_or(&default_url);

    let response = reqwest::get(url)
        .await
        .expect("Failed to fetch images")
        .json::<Vec<Group>>()
        .await
        .expect("Failed to parse images");

    Ok(response)
}

pub async fn download_template(tmp_dir: &Path, client: &Client, template: &Template, pb: &ProgressBar) -> anyhow::Result<PathBuf> {
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.bold.dim} {bar} {percent}% [{elapsed_precise}] {bytes}/{total_bytes} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Downloading {}", template.name));

    let retry_count = 3;
    let mut attempt = 0;

    let response = loop {
        let response = client.get(&template.link).send().await?;
        if response.status().is_success() || attempt >= retry_count {
            break response;
        }
        attempt += 1;
        sleep(std::time::Duration::from_secs(attempt)).await;
    };

    if !response.status().is_success() {
        return Err(anyhow!(format!("Failed to download {} after {} attempts", template.name, attempt)));
    }

    let compression_ext = {
        let path = Path::new(response.url().path());
        let extension = path.extension()
            .and_then(|p| p.to_str())
            .and_then(|p| Some(p.to_lowercase()));

        match extension {
            Some(ext) => {
                // if &ext == "vma" {
                //     "" // indicative of no compression used
                // }

                format!(".{}", ext)
            }
            None => panic!("{}", format!("{}: Unsupported image format", template.name)) // we can't handle non-vma files
        }
    };

    pb.set_length(response.content_length().unwrap_or(0));

    let temp_file_path = tmp_dir.join(format!("vzdump-qemu-{}.vma{}", template.vmid, compression_ext));
    let mut file = File::create(&temp_file_path).await?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let data = chunk.unwrap();
        file.write_all(&data).await?;
        pb.inc(data.len() as u64);
    }

    pb.finish_with_message(format!("Downloaded {}", template.name));

    Ok(temp_file_path)
}
