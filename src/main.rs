use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::{Client};
use serde::{Deserialize, Serialize};
use tokio::task;
use tokio::time::{sleep, Duration};
use tokio::{fs::File as TokioFile, io::AsyncWriteExt};

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

#[tokio::main]
async fn main() -> Result<(), ()> {
    let data = r#"
    [
        {
            "name": "Ubuntu",
            "templates": [
                { "name": "Ubuntu 18.04", "vmid": 1000, "link": "https://cdn.convoypanel.com/ubuntu/ubuntu-18-04-amd64.vma.zst" },
                { "name": "Ubuntu 20.04", "vmid": 1001, "link": "https://cdn.convoypanel.com/ubuntu/ubuntu-20-04-amd64.vma.zst" },
                { "name": "Ubuntu 22.04", "vmid": 1002, "link": "https://cdn.convoypanel.com/ubuntu/ubuntu-22-04-amd64.vma.zst" }
            ]
        },
        {
            "name": "Windows Server",
            "templates": [
                { "name": "Windows Server 2019", "vmid": 2000, "link": "https://cdn.convoypanel.com/windows/windows-2019-datacenter-amd64.vma.zst" },
                { "name": "Windows Server 2022", "vmid": 2001, "link": "https://cdn.convoypanel.com/windows/windows-2022-datacenter-amd64.vma.zst" }
            ]
        },
        {
            "name": "CentOS",
            "templates": [
                { "name": "CentOS 7", "vmid": 3000, "link": "https://cdn.convoypanel.com/centos/centos-7-amd64.vma.zst" },
                { "name": "CentOS 8", "vmid": 3001, "link": "https://cdn.convoypanel.com/centos/centos-8-amd64.vma.zst" }
            ]
        },
        { "name": "Debian", "templates": [{ "name": "Debian 11", "vmid": 4000, "link": "https://cdn.convoypanel.com/debian/debian-11-amd64.vma.zst" }] },
        { "name": "Rocky Linux", "templates": [{ "name": "Rocky Linux 8", "vmid": 5000, "link": "https://cdn.convoypanel.com/rocky-linux/rocky-linux-8-amd64.vma.zst" }] }
    ]
    "#;

    let groups: Vec<Group> = serde_json::from_str(data).unwrap();

    let multi_progress = MultiProgress::new();

    let mut tasks = vec![];

    for group in groups {
        for template in group.templates {
            let pb = multi_progress.add(ProgressBar::new(100));
            let task = task::spawn(download_and_install_template(template, pb));

            tasks.push(task);
        }
    }

    join_all(tasks).await;
    Ok(())
}

async fn download_and_install_template(
    template: Template,
    progress_bar: ProgressBar,
) -> Result<(), ()> {
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.bold.dim} {bar} {percent}% [{elapsed_precise}] {msg}")
            .unwrap(),
    );

    download_template(&template, &progress_bar).await.expect("Couldn't download template");

    progress_bar.set_length(100);
    progress_bar.set_message(format!("Installing {}", template.name));
    progress_bar.set_position(0);

    for _j in 0..100 {
        progress_bar.inc(1);
        sleep(Duration::from_millis(500)).await;
    }
    progress_bar.finish();

    Ok(())
}

async fn download_template(template: &Template, progress_bar: &ProgressBar) -> Result<String, ()> {
    progress_bar.set_message(format!("Downloading {}", template.name));
    let client = Client::new();
    let mut response = client.get(&template.link).send().await.unwrap(); // TODO: need to add retry ability bc network errors aren't uncommon
    let content_length = response
        .content_length()
        .expect("Couldn't get content length");

    progress_bar.set_length(content_length);

    let mut buffer: Vec<u8> = Vec::new();

    let file_name = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .expect("Couldn't get template's file name")
        .to_string();

    let mut temp_file_name = file_name.clone();
    temp_file_name.push_str(".tmp");

    let mut file = TokioFile::create(&temp_file_name)
        .await
        .expect("Couldn't create file");
    while let Some(chunk) = response.chunk().await.unwrap() {
        let data = chunk.to_vec();
        buffer.extend(data.iter());
        progress_bar.inc(data.len() as u64);
        file.write_all(&data).await.expect("Couldn't write file");
    }

    std::fs::rename(&temp_file_name, &file_name).expect("Couldn't rename file");

    progress_bar.finish_with_message(format!("Downloaded {}", template.name));
    Ok(file_name)
}
