use crate::installer::download_and_install_templates;
use crate::util::{get_storage_location, show_branding};

mod util;
mod downloader;
mod installer;

#[tokio::main]
async fn main() -> Result<(), ()> {
    show_branding();

    let storage_volume = get_storage_location();

    let tmp_dir = tempfile::tempdir().unwrap();
    let tmp_path = tmp_dir.path();

    dbg!(&tmp_path);

    tokio::select! {
        _ = download_and_install_templates(&tmp_path, &storage_volume) => {},
        _ = tokio::signal::ctrl_c() => {
            println!("Received Ctrl-C, exiting");
        }
    }

    tmp_dir.close().unwrap();

    Ok(())
}