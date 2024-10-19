use std::{env::var, fs::read_dir, process::Command};

fn main() {
    println!("cargo::rerun-if-changed=src/sprites");
    let paths = read_dir("./src/sprites")
        .unwrap()
        .map(|entry| entry.unwrap().path().to_str().unwrap().to_string());
    for path in paths {
        assert!(Command::new(include_str!("src/aseprite_path").trim())
            .arg("-b")
            .arg(&path)
            .arg("--save-as")
            .arg(format!(
                "{}/{}.png",
                var("OUT_DIR").unwrap(),
                path.split(['/', '.']).rev().nth(1).unwrap()
            ))
            .spawn()
            .unwrap()
            .wait()
            .unwrap()
            .success());
    }
}
