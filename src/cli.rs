// MIT License

// Copyright (c) 2020 Andrew Plaza

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use derive_builder::Builder;
use clap::load_yaml;
use clap::App;

#[derive(Default, Builder, Debug)]
pub struct Configuration {
    pub ignore_target: bool,
    pub git_repo: String,
    pub new_repo: String,
    pub rev: Option<String>,
    pub branch: Option<String>
}

pub fn parse_args() -> Configuration {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let mut config = ConfigurationBuilder::default();

    let ignore_target = matches.is_present("ignore_target");
    config.ignore_target(ignore_target);

    let git_repo = matches.value_of("git_repo").expect("did not find git repository to use");
    config.git_repo(git_repo.to_string());

    let new_repo = matches.value_of("new_repo").expect("did not find any repository to use");
    config.new_repo(new_repo.to_string());

    let rev = matches.value_of("rev");
    config.rev(rev.map(|s| s.to_string()));

    let branch = matches.value_of("branch");
    config.branch(branch.map(|s| s.to_string()));

    config.build().expect("Could not build config")
}
