// build.rs

use std::{env, path::Path};

fn build_nftset() -> anyhow::Result<()> {
    let target = env::var("TARGET")?;

    if !target.contains("linux") {
        return Ok(());
    }

    let mut build = cc::Build::new();
    build
        .file("include/nftset.c")
        .static_flag(true)
        .warnings(false);

    if target.ends_with("-musl") {
        let target_dir = env::var_os("OUT_DIR").unwrap();
        let musl_root = Path::new(&target_dir);
        let target = target.replace("unknown-linux", "linux");
        let include_dir = musl_root.join(&format!("{target}-native")).join("include");
        if !musl_root.exists() {
            std::fs::create_dir_all(musl_root)?;
        }
        
        let file = musl_root.join(&format!("{target}-native.tgz"));
        if !file.exists() {
            std::process::Command::new("curl")
                .args(["-OL", &format!("https://musl.cc/{target}-native.tgz")])
                .current_dir(musl_root)
                .output()
                .expect(&format!("download https://musl.cc/{target}-native.tgz failed"));
        }

        if !include_dir.exists() {
            std::process::Command::new("tar")
                .args(["-xzf", &format!("{target}-native.tgz")])
                .current_dir(musl_root)
                .output()
                .expect(&format!("untar {target}-native.tgz failed"));
        }

        build.include(include_dir.as_os_str()); // https://musl.cc/x86_64-linux-musl-native.tgz
    }

    build.compile("nftset");

    bindgen::Builder::default()
        .header("include/nftset.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("src/ffi/nftset.rs")
        .unwrap();

    Ok(())
}

fn main() -> anyhow::Result<()> {
    build_nftset()?;
    Ok(())
}
