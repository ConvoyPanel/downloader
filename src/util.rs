use std::path::PathBuf;
use dialoguer::Input;

pub fn show_branding() {
    println!(r#"
  _____
 / ___/__  ___ _  _____  __ __
/ /__/ _ \/ _ \ |/ / _ \/ // /
\___/\___/_//_/___/\___/\_, /
                       /___/
    "#);
    println!(
        "Convoy Templates Downloader\nVersion: {}\n",
        env!("CARGO_PKG_VERSION")
    );
    println!("View the source code at https://github.com/convoypanel/downloader\n\n\n");
}

pub fn get_storage_location() -> String {
    let location = Input::new()
        .with_prompt("Please enter the storage volume that you want to import the templates into")
        .default("local-lvm".into())
        .interact_text()
        .unwrap();

    location
}

pub fn is_vmid_used(vmid: &i32) -> bool {
    let path = PathBuf::from("/etc/pve/qemu-server/");
    let file_path = path.join(format!("{}.conf", vmid));
    file_path.exists()
}
