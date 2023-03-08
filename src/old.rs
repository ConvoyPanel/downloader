use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::io::Cursor;
use tokio::task::JoinHandle;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::time::{sleep, Duration};
use tokio::task;
use std::sync::Arc;


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

pub type Groups = Vec<Group>;

async fn download(template: Template) -> Result<(), ()> {

    println!("heeeya");

    Ok(())
}

// #[tokio::main]
// async fn main() {
//     let data = r#"
//     [
//         {
//             "name": "Ubuntu",
//             "templates": [
//                 { "name": "Ubuntu 18.04", "vmid": 1000, "link": "https://cdn.convoypanel.com/ubuntu/ubuntu-18-04-amd64.vma.zst" },
//                 { "name": "Ubuntu 20.04", "vmid": 1001, "link": "https://cdn.convoypanel.com/ubuntu/ubuntu-20-04-amd64.vma.zst" },
//                 { "name": "Ubuntu 22.04", "vmid": 1002, "link": "https://cdn.convoypanel.com/ubuntu/ubuntu-22-04-amd64.vma.zst" }
//             ]
//         },
//         {
//             "name": "Windows Server",
//             "templates": [
//                 { "name": "Windows Server 2019", "vmid": 2000, "link": "https://cdn.convoypanel.com/windows/windows-2019-datacenter-amd64.vma.zst" },
//                 { "name": "Windows Server 2022", "vmid": 2001, "link": "https://cdn.convoypanel.com/windows/windows-2022-datacenter-amd64.vma.zst" }
//             ]
//         },
//         {
//             "name": "CentOS",
//             "templates": [
//                 { "name": "CentOS 7", "vmid": 3000, "link": "https://cdn.convoypanel.com/centos/centos-7-amd64.vma.zst" },
//                 { "name": "CentOS 8", "vmid": 3001, "link": "https://cdn.convoypanel.com/centos/centos-8-amd64.vma.zst" }
//             ]
//         },
//         { "name": "Debian", "templates": [{ "name": "Debian 11", "vmid": 4000, "link": "https://cdn.convoypanel.com/debian/debian-11-amd64.vma.zst" }] },
//         { "name": "Rocky Linux", "templates": [{ "name": "Rocky Linux 8", "vmid": 5000, "link": "https://cdn.convoypanel.com/rocky-linux/rocky-linux-8-amd64.vma.zst" }] }
//     ]
//     "#;

//     let groups: Groups = serde_json::from_str(data).unwrap();

//     let mut tasks: Vec<JoinHandle<()>> = vec![];

//     let mut pbs: Vec<ProgressBar> = vec![];

//     for group in groups {
//         println!("Group: {}", group.name);
//         for template in group.templates {
//             let mut pb = ProgressBar::new(100);
//             pb.set_style(
//                 ProgressStyle::default_bar()
//                     .template("{spinner} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})").unwrap()
//                     .progress_chars("#>-"),
//             );
//             pbs.push(pb);

//             tasks.push(tokio::spawn(async {
//                 //download(template).await.unwrap();
//                 let pb = pb.clone();
//                 for _ in 0..1000 {
//                     pb.inc(1);
//                     sleep(Duration::from_secs(4)).await;
//                 }
//             }));
//         }
//     }

//     join_all(tasks).await;
// }

// #[tokio::main]
// async fn main() {
//     let pbs = vec![
//         ProgressBar::new(100),
//         ProgressBar::new(100),
//         ProgressBar::new(100),
//     ];
//     let pbs = Arc::new(pbs);

//     let mut tasks = vec![];
//     for (i, pb) in pbs.iter().enumerate() {
//         let pbs = pbs.clone();
//         let task = task::spawn(async move {
//             let pb = &pbs[i];
//             pb.set_style(
//                 ProgressStyle::default_bar()
//                     .template("{prefix:.bold.dim} {bar} {percent}% [{elapsed_precise}]").unwrap(),
//             );
//             let prefix = format!("Task {}", i + 1);
//             pb.set_prefix(&prefix);
//             for j in 0..100 {
//                 pb.inc(1);
//                 sleep(Duration::from_millis(100)).await;
//             }
//             pb.finish();
//         });
//         tasks.push(task);
//     }

//     for task in tasks {
//         task.await.unwrap();
//     }
// }

#[tokio::main]
async fn main() {
    let pbs = vec![
        ProgressBar::new(100),
        ProgressBar::new(100),
        ProgressBar::new(100),
    ];
    let pbs = Arc::new(pbs);

    let mut tasks = vec![];
    for (i, pb) in pbs.iter().enumerate() {
        let prefix = format!("Task {}", i + 1);
        let prefix = Arc::new(prefix);
        let task = task::spawn(async move {
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{prefix:.bold.dim} {bar} {percent}% [{elapsed_precise}]").unwrap(),
            );
            pb.set_prefix((*prefix).to_owned());
            for j in 0..100 {
                pb.inc(1);
                sleep(Duration::from_millis(100)).await;
            }
            pb.finish();
        });
        tasks.push(task);
    }

    for task in tasks {
        task.await.unwrap();
    }
}