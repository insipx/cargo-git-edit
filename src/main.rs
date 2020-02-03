// MIT License

// Copyright (c) 2020 Andrew Plaza

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use globset::Glob;
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use toml_edit::{value, Document, InlineTable, Item, Table, Value};
use walkdir::WalkDir;

use self::cli::Configuration;
use crate::err::Error;

mod cli;
mod err;

fn main() -> Result<(), Error> {
    let app_config = cli::parse_args();
    let files = get_cargo_configs(app_config.ignore_target);
    let cargo_configs = read_configs(&files);
    let mutator = MutateGitDeps::new(app_config);
    mutator.run(cargo_configs)?;

    Ok(())
}

struct MutateGitDeps {
    app_config: Configuration,
}

impl MutateGitDeps {
    pub fn new(app_config: Configuration) -> Self {
        Self { app_config }
    }

    pub fn run(
        self, mut cargo_configs: HashMap<String, Document>,
    ) -> Result<(), Error> {
        for (path, config) in cargo_configs.iter_mut() {
            if config.as_table().contains_table("dependencies") {
                self.mutate_git_deps(config, "dependencies")?;
            }
            if config.as_table().contains_table("dev-dependencies") {
                self.mutate_git_deps(config, "dev-dependencies")?;
            }
        }

        for (path, conf) in cargo_configs.iter() {
            println!("{}", conf.to_string());
        }
        Ok(())
    }

    fn mutate_git_deps(
        &self, doc: &mut Document, table: &str,
    ) -> Result<(), Error> {
        let mut keys: Vec<String> = Vec::new();
        dbg!(table);
         // check for table or inline table
         // otherwise fails
        let dep_table = doc
            .as_table()
            .get(table)
            .ok_or(Error::NotFound("Table".to_string()))?
            .as_table()
            .ok_or(Error::NotFound("Convert Table".to_string()))?;

        for (k, v) in dep_table.iter() {
            dbg!(v);
            if v.as_value()
                .ok_or(Error::NotFound("dep Value".to_string()))?
                .is_inline_table()
            {
                let val = v
                    .as_value()
                    .ok_or(Error::NotFound("val Value".to_string()))?
                    .as_inline_table()
                    .ok_or(Error::NotFound("Inline Table".to_string()))?;

                if val.contains_key("git") {
                    keys.push(k.to_string());
                }
            }
        }
        let mut dep_table = doc
            .as_table_mut()
            .entry(table)
            .as_table_mut()
            .ok_or(Error::NotFound("Convert Table".to_string()))?;

        self.change_selected_values(dep_table, keys)?;
        Ok(())
    }

    /// internal api to change values based on the values inputted by user
    fn change_selected_values(
        &self, dep: &mut Table, keys: Vec<String>,
    ) -> Result<(), Error> {
        for key in keys.iter() {
            let dep: &mut InlineTable = dep
                .entry(key)
                .as_value_mut()
                .ok_or(Error::NotFound("git Value".to_string()))?
                .as_inline_table_mut()
                .ok_or(Error::NotFound("Inline Table".to_string()))?;
            dep.remove("git");
            dep.get_or_insert("git", self.app_config.new_repo.clone());

            if let Some(r) = &self.app_config.rev {
                if dep.contains_key("rev") {
                    dep.remove("rev");
                }
                dep.get_or_insert("rev", r.to_string());
                if dep.contains_key("branch") {
                    dep.remove("branch");
                }
            }
            if let Some(b) = &self.app_config.branch {
                if dep.contains_key("branch") {
                    dep.remove("branch");
                }
                dep.get_or_insert("branch", b.to_string());
                if dep.contains_key("rev") {
                    dep.remove("rev");
                }
            }
        }
        Ok(())
    }
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
