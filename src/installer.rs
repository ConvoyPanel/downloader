use std::path::Path;
use std::process::Stdio;

use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::Client;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::task::JoinHandle;

use crate::downloader;
use crate::downloader::{download_template, Template};
use crate::util::is_vmid_used;

pub async fn install_template(
    storage_volume: &String,
    template: &Template,
    image: &Path,
    pb: &ProgressBar) -> anyhow::Result<()>
{
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.bold.dim} {bar} {percent}% [{elapsed_precise}] {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Installing {}", template.name));
    pb.set_length(100);
    pb.set_position(0);

    let restoration = Command::new("qmrestore")
        .args(&[
            image.to_str().unwrap(),
            template.vmid.to_string().as_str(),
            "-storage",
            &storage_volume,
        ])
        .stdout(Stdio::piped())
        .spawn();

    if restoration.is_err() {
        pb.finish_with_message(format!("Failed to restore {}", template.name));
        return Err(anyhow::anyhow!("Failed to restore {}", template.name));
    }


    let stdout = restoration.unwrap().stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();

    while let Some(line) = reader.next_line().await.unwrap() {
        let re = Regex::new(r"progress (\d+)%").unwrap();
        if let Some(captures) = re.captures(&line) {
            let percent = captures.get(1).unwrap().as_str().parse::<u64>().unwrap();
            pb.set_position(percent);
        }
    }

    pb.finish_with_message(format!("Installed {}", template.name));

    Ok(())
}

pub async fn download_and_install_templates(tmp_dir: &Path, storage_volume: &String) -> anyhow::Result<()> {
    let client = Client::new();
    let groups = downloader::fetch_templates().await?;
    let mpb = MultiProgress::new();
    let mut tasks = vec![];

    for group in groups {
        for template in group.templates {
            let pb = mpb.add(ProgressBar::new(100));

            if !is_vmid_used(&template.vmid) {
                let client = client.clone();
                let storage_volume = storage_volume.clone();
                let tmp_dir = tmp_dir.to_path_buf();
                let task: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
                    let file_name = download_template(&tmp_dir, &client, &template, &pb).await?;
                    install_template(&storage_volume, &template, &file_name, &pb).await?;

                    Ok(())
                });

                tasks.push(task);
            } else {
                pb.finish_with_message(format!("vmid {} is taken for {}", template.vmid, template.name));
            }
        }
    }

    join_all(tasks).await;
    Ok(())
}
