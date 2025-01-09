pub fn get_project_root() -> std::path::PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .expect("Failed to run cargo locate-project");

    let cargo_toml =
        String::from_utf8(output.stdout).expect("Failed to parse cargo locate-project output");

    Path::new(&cargo_toml.trim())
        .parent()
        .expect("Failed to find project root")
        .to_path_buf()
}
