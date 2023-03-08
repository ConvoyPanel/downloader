use futures::future::join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{self, Read};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::task;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), ()> {
    let urls = vec![
        "https://www.example.com/file1.txt".to_string(),
        "https://www.example.com/file2.txt".to_string(),
        "https://www.example.com/file3.txt".to_string(),
    ];

    let multi_progress = MultiProgress::new();

    let mut tasks = vec![];

    let filenames = vec![
        "file1.txt".to_string(),
        "file2.txt".to_string(),
        "file3.txt".to_string(),
    ];

    let v2 = filenames.iter();

    for (_url, filename) in urls.iter().zip(v2) {
        let pb = multi_progress.add(ProgressBar::new(100));
        let filename = filename.to_owned();
        let task = task::spawn(async move {
            pb.set_message(format!("Downloading {}", filename));
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{prefix:.bold.dim} {bar} {percent}% [{elapsed_precise}]")
                    .unwrap(),
            );
            for _j in 0..100 {
                pb.inc(1);
                sleep(Duration::from_millis(500)).await;
            }
            pb.finish();
        });
        tasks.push(task);
    }

    join_all(tasks).await;
    Ok(())
}
