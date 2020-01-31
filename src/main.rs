use globset::Glob;
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use toml_edit::{value, Document};
use walkdir::WalkDir;

fn main() {
    let files = get_cargo_configs(false);
    let configs = read_configs(&files);
    configs.iter().for_each(|(_k, f)| println!("{:?}", f));

    // look for key in each toml file that corresponds to user key and  modify
    // values according to user input
}



/// Read a series of File Paths that correspond to Cargo.toml files
///
/// # Panics
/// panics on file open I/O error, if reading the file fails,
/// if the Cargo.toml is not valid toml, or if the file path
/// is not valid UTF-8
fn read_configs(files: &Vec<PathBuf>) -> HashMap<String, Document> {
    let mut configs = HashMap::new();

    for c in files.iter() {
        let mut f = File::open(&c).expect("Could not open cargo.toml");
        let mut buffer = String::new();
        f.read_to_string(&mut buffer).expect("Reading file failed");
        let toml = buffer.parse::<Document>().expect("Invalid doc");
        configs.insert(
            c.to_str().expect("String not valid unicode").to_string(),
            toml,
        );
    }
    configs
}

/// Gets Cargo.tomls, optionally ignoring the /target/ directory, from a
/// workspace on-disk
fn get_cargo_configs(ignore_target: bool) -> Vec<PathBuf> {
    WalkDir::new("./")
        .into_iter()
        .filter_map(|e| e.ok().map(|f| f.into_path())) // converts to option, disregards err
        .filter(|p| filter_cargo_configs(p.as_path(), ignore_target))
        .map(|p| p.into())
        .collect::<Vec<PathBuf>>()
}

fn filter_cargo_configs<P: AsRef<Path>>(path: P, ignore_target: bool) -> bool {
    let glob = Glob::new("**/Cargo.toml")
        .expect("Failed")
        .compile_matcher();

    let ignore_target_glob = Glob::new("**/target/**/Cargo.toml")
        .expect("Failed")
        .compile_matcher();

    match ignore_target {
        true => glob.is_match(&path) && !ignore_target_glob.is_match(&path),
        false => glob.is_match(&path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATHS: [&str; 9] = [
        "/home/lain/code/chisa/Cargo.toml",
        "/home/lain/code/chisa/src/main.rs",
        "/home/lain/code/eiri/Cargo.toml",
        "/home/lain/code/eiri/target/debug/wbuild/Cargo.toml",
        "./target/debug/wbuild/Cargo.toml",
        "Cargo.toml",
        "./Cargo.toml",
        "./code/kami/Cargo.toml",
        "./code/kami/src/lib.rs",
    ];

    #[test]
    fn should_get_cargo_configs() {
        let matches = PATHS
            .iter()
            .map(|p| filter_cargo_configs(p, true))
            .collect::<Vec<bool>>();
        assert_eq!(
            matches,
            vec![true, false, true, false, false, true, true, true, false]
        );
    }

    #[test]
    fn should_include_target() {
        let matches = PATHS
            .iter()
            .map(|p| filter_cargo_configs(p, false))
            .collect::<Vec<bool>>();
        assert_eq!(
            matches,
            vec![true, false, true, true, true, true, true, true, false]
        );
    }
}
