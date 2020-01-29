use globset::Glob;
use walkdir::WalkDir;

fn main() {

}


fn get_cargo_configs(ignore_target: bool) -> Vec<PathBuf> {
    let glob = Glob::new("./**/Cargo.toml").expect("Failed").compile_matcher();
    let ignore_glob = Glob::new("./target/**/Cargo.toml").expect("Failed").compile_matcher();
    for entry in WalkDir::new("./")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| glob.is_match(f.path()))
        .filter(|f| ignore_target && !ignore_glob.is_match(f.path()))
    {
        println!("{:?}", entry.path().display())
    }

}
