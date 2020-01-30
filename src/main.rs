use globset::{Glob, GlobMatcher};
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() {
    let confs = get_cargo_configs(false);
    confs.iter().for_each(|p| {
        println!("{}", p.display());
    });
}

/// Gets Cargo.tomls, optionally ignoring the /target/ directory, from a
/// workspace on-disk
fn get_cargo_configs(ignore_target: bool) -> Vec<PathBuf> {
    let glob = Glob::new("**/Cargo.toml")
        .expect("Failed")
        .compile_matcher();

    let ignore_target_glob = Glob::new("./target/**/Cargo.toml")
        .expect("Failed")
        .compile_matcher();

    WalkDir::new("./")
        .into_iter()
        .filter_map(|e| e.ok()) // converts to option, disregards err
        .filter(|f| glob.is_match(f.path()))
        .filter(|f| {
            if ignore_target && ignore_target_glob.is_match(f.path()) {
                false
            } else {
                true
            }
        })
        .map(|f| f.into_path())
        .collect::<Vec<PathBuf>>()
}

fn edit_files() {


}
