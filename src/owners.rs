// cown - a small CODEOWNERS tool
// Copyright (C) 2022 Ben Bader
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fs;
use std::io::{self, BufRead};
use std::path;

pub struct OwnersFile {
    // The absolute path on disk of this CODEOWNERS file.
    pub path: path::PathBuf,

    // All
    pub rules: Vec<Rule>,
}

impl OwnersFile {
    pub fn new(path: path::PathBuf, rules: Vec<Rule>) -> Self {
        Self { path, rules }
    }

    pub fn try_parse(file: path::PathBuf) -> io::Result<OwnersFile> {
        let handle = fs::File::open(&file)?;
        let buf = io::BufReader::new(handle);

        let mut rules = Vec::new();
        for (line_number, line) in buf.lines().enumerate() {
            let line = line?;
            if let Some(rule) = Rule::try_parse(&line, line_number + 1) {
                rules.push(rule);
            }
        }
        rules.reverse();

        Ok(Self::new(file, rules))
    }

    pub fn owner_for<P: AsRef<path::Path>>(&self, path: P) -> Option<&Vec<String>> {
        let root = self.root();

        let mut path = path.as_ref();
        if path.starts_with(root) {
            path = path.strip_prefix(root).unwrap();
        }

        self.rules
            .iter()
            .filter(|r| r.matches_file(path))
            .map(|r| &r.owners)
            .next()
    }

    fn root(&self) -> &path::Path {
        let dir = self.path.parent().unwrap();
        if dir.ends_with(".github") || dir.ends_with("docs") {
            dir.parent().unwrap()
        } else {
            dir
        }
    }
}

pub struct Rule {
    pub pattern: glob::Pattern,
    pub owners: Vec<String>,
    pub line_number: usize,
}

impl Rule {
    pub fn try_parse(line: &str, line_number: usize) -> Option<Self> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }

        let segments: Vec<&str> = line.split_whitespace().collect();

        Self::parse_pattern(segments.first().unwrap()).map(|pat| Self {
            pattern: pat,
            owners: segments.iter().skip(1).map(|&s| s.to_string()).collect(),
            line_number,
        })
    }

    fn parse_pattern(text: &str) -> Option<glob::Pattern> {
        let prefixed = if text.starts_with('*') || text.starts_with('/') {
            text.to_owned()
        } else {
            format!("**/{}", text)
        };

        let mut normalized = prefixed.trim_start_matches('/').to_string();
        if normalized.ends_with('/') {
            normalized.push_str("**");
        }

        match glob::Pattern::new(&normalized) {
            Ok(pat) => Some(pat),
            Err(err) => {
                eprintln!(
                    "ERROR: Invalid CODEOWNERS pattern '{}': {}",
                    normalized, err
                );
                None
            }
        }
    }

    pub fn matches_file(&self, path: &path::Path) -> bool {
        self.pattern.matches_path(path)
    }
}
