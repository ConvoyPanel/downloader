use dialoguer::Input;
use futures::future::join_all;
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::task;
use tokio::{fs::File as TokioFile, io::AsyncWriteExt};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use regex::Regex;

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
    println!(
        "
 ██████  ██████  ███    ██ ██    ██  ██████  ██    ██
██      ██    ██ ████   ██ ██    ██ ██    ██  ██  ██
██      ██    ██ ██ ██  ██ ██    ██ ██    ██   ████
██      ██    ██ ██  ██ ██  ██  ██  ██    ██    ██
 ██████  ██████  ██   ████   ████    ██████     ██
    "
    );
    println!(
        "Convoy Templates Downloader\nVersion: {}\n",
        env!("CARGO_PKG_VERSION")
    );
    println!("View the source code at https://github.com/convoypanel/downloader\n\n\n");

    let location = get_storage_location();

    let data = r#"
    [
        {
            "name": "Ubuntu",
            "templates": [
                { "name": "Ubuntu 18.04", "vmid": 1000, "link": "https://images.cdn.convoypanel.com/ubuntu/ubuntu-18-04-amd64.vma.zst" },
                { "name": "Ubuntu 20.04", "vmid": 1001, "link": "https://images.cdn.convoypanel.com/ubuntu/ubuntu-20-04-amd64.vma.zst" },
                { "name": "Ubuntu 22.04", "vmid": 1002, "link": "https://images.cdn.convoypanel.com/ubuntu/ubuntu-22-04-amd64.vma.zst" }
            ]
        },
        {
            "name": "Windows Server",
            "templates": [
                { "name": "Windows Server 2019", "vmid": 2000, "link": "https://images.cdn.convoypanel.com/windows/windows-2019-datacenter-amd64.vma.zst" },
                { "name": "Windows Server 2022", "vmid": 2001, "link": "https://images.cdn.convoypanel.com/windows/windows-2022-datacenter-amd64.vma.zst" }
            ]
        },
        {
            "name": "CentOS",
            "templates": [
                { "name": "CentOS 7", "vmid": 3000, "link": "https://images.cdn.convoypanel.com/centos/centos-7-amd64.vma.zst" },
                { "name": "CentOS 8", "vmid": 3001, "link": "https://images.cdn.convoypanel.com/centos/centos-8-amd64.vma.zst" }
            ]
        },
        { "name": "Debian", "templates": [{ "name": "Debian 11", "vmid": 4000, "link": "https://images.cdn.convoypanel.com/debian/debian-11-amd64.vma.zst" }] },
        { "name": "Rocky Linux", "templates": [{ "name": "Rocky Linux 8", "vmid": 5000, "link": "https://images.cdn.convoypanel.com/rocky-linux/rocky-linux-8-amd64.vma.zst" }] }
    ]
    "#;

    let groups: Vec<Group> = serde_json::from_str(data).unwrap();

    let multi_progress = MultiProgress::new();

    let mut tasks = vec![];

    for group in groups {
        for template in group.templates {
            let pb = multi_progress.add(ProgressBar::new(100));
            let task = task::spawn(download_and_install_template(location.clone(), template, pb));

            tasks.push(task);
        }
    }

    join_all(tasks).await;
    Ok(())
}

fn get_storage_location() -> String {
    let location = Input::new()
        .with_prompt("Please enter the storage volume that you want to import the templates into")
        .default("local-lvm".into())
        .interact_text()
        .unwrap();

    location
}

async fn download_and_install_template(
    location: String,
    template: Template,
    progress_bar: ProgressBar,
) -> Result<(), ()> {

    let check_vm_exists_command = Command::new("qm")
        .args(&["status", &template.vmid.to_string()])
        .output()
        .await
        .unwrap();

    if check_vm_exists_command.status.success() {
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{prefix:.bold.dim} {bar} {percent}% [{elapsed_precise}] {msg}")
                .unwrap(),
        );

        progress_bar.finish_with_message(format!("{} (vmid: {}) already exists", template.name, template.vmid));
        return Ok(());
    }

    let file_name = download_template(&template, &progress_bar)
        .await
        .expect("Couldn't download template");

    progress_bar.set_length(100);
    progress_bar.set_message(format!("Installing {}", template.name));
    progress_bar.set_position(0);

    install_template(file_name, location, &template, &progress_bar).await;

    Ok(())
}

async fn install_template(
    file_name: String,
    location: String,
    template: &Template,
    progress_bar: &ProgressBar,
) {
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.bold.dim} {bar} {percent}% [{elapsed_precise}] {msg}")
            .unwrap(),
    );

    let new_file_name = format!("vzdump-qemu-{}.vma.zst", template.vmid);
    tokio::fs::rename(&file_name, &new_file_name)
        .await
        .expect("Couldn't rename file");

    let mut child_process = Command::new("qmrestore")
        .args(&[
            &new_file_name,
            &template.vmid.to_string(),
            &"-storage".to_string(),
            &location,
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = child_process.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout).lines();

    while let Some(line) = reader.next_line().await.unwrap() {
        let re = Regex::new(r"progress (\d+)%").unwrap();
        if let Some(captures) = re.captures(&line) {
            let percent = captures.get(1).unwrap().as_str().parse::<u64>().unwrap();
            progress_bar.set_position(percent);
        }
    }

    // delete file
    tokio::fs::remove_file(&new_file_name)
        .await
        .expect("Couldn't delete file");

    progress_bar.finish_with_message(format!("Installed {}", template.name))
}

async fn download_template(template: &Template, progress_bar: &ProgressBar) -> Result<String, ()> {
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.bold.dim} {bar} {percent}% [{elapsed_precise}] {bytes}/{total_bytes} {msg}")
            .unwrap(),
    );
    progress_bar.set_message(format!("Downloading {}", template.name));

    let client = Client::new();
    let response = client.get(&template.link).send().await.unwrap(); // TODO: need to add retry ability bc network errors aren't uncommon
    let content_length = response
        .content_length()
        .expect("Couldn't get content length");

    progress_bar.set_length(content_length);

    let file_name = response
        .url()
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .expect("Couldn't get template's file name")
        .to_string();

    let mut content = response.bytes_stream();

    let mut temp_file_name = file_name.clone();
    temp_file_name.push_str(".tmp");

    let mut file = TokioFile::create(&temp_file_name)
        .await
        .expect("Couldn't create file");

    while let Some(chunk) = content.next().await {
        let data = chunk.unwrap();
        progress_bar.inc(data.len() as u64);
        file.write_all(&data).await.expect("Couldn't write file");
    }

    tokio::fs::rename(&temp_file_name, &file_name)
        .await
        .expect("Couldn't rename file");

    progress_bar.finish_with_message(format!("Downloaded {}", template.name));
    Ok(file_name)
}

// cleanup
// async fn cleanup() -> Result<(), std::io::Error> {
//     let dir = ".";
//     let mut entries = tokio::fs::read_dir(dir).await?;

//     while let Some(entry) = entries.next_entry().await? {
//         let path = entry.path();
//         if let Some(ext) = path.extension() {
//             if ext == "zst" || ext == "tmp" {
//                 let file_name = path.file_name().unwrap().to_str().unwrap();
//                 if file_name.ends_with(".vma.zst") || file_name.ends_with(".vma.zst.tmp") {
//                     tokio::fs::remove_file(path).await?;
//                 }
//             }
//         }
//     }

//     Ok(())
// }