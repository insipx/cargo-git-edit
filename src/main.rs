use globset::{Glob, GlobMatcher};
use std::path::{PathBuf, Path};
use walkdir::WalkDir;

fn main() {
    let confs = get_cargo_configs(false);

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
        false => glob.is_match(&path)
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
        "./code/kami/src/lib.rs"
    ];

    #[test]
    fn should_get_cargo_configs() {
        let matches = PATHS.iter()
             .map(|p| filter_cargo_configs(p, true))
             .collect::<Vec<bool>>();
        assert_eq!(matches, vec![true, false, true, false, false, true, true, true, false]);
    }

    #[test]
    fn should_include_target() {
        let matches = PATHS.iter()
                           .map(|p| filter_cargo_configs(p, false))
                           .collect::<Vec<bool>>();
        assert_eq!(matches, vec![true, false, true, true, true, true, true, true, false]);
    }
}
