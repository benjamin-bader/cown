// owns - a small CODEOWNERS tool
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
    pub fn new(path: path::PathBuf) -> Self {
        Self {
            path,
            rules: vec![],
        }
    }

    pub fn try_parse(file: path::PathBuf) -> io::Result<Option<OwnersFile>> {
        let handle = fs::File::open(&file)?;
        let buf = io::BufReader::new(handle);

        let mut owners_file = Self::new(file);
        for line in buf.lines() {
            let line = line?;
            if let Some(rule) = Rule::try_parse(&line) {
                owners_file.add_rule(rule);
            }
        }

        owners_file.rules.reverse();

        Ok(Some(owners_file))
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    pub fn owner_for<P: AsRef<path::Path>>(&self, path: P) -> Option<&Vec<String>> {
        self.rules
            .iter()
            .filter(|r| r.matches_file(path.as_ref()))
            .map(|r| &r.owners)
            .next()
    }
}

pub struct Rule {
    pub pattern: glob::Pattern,
    pub owners: Vec<String>,
}

impl Rule {
    pub fn try_parse(line: &str) -> Option<Self> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }

        let segments: Vec<&str> = line.split_whitespace().collect();

        Self::parse_pattern(segments.first().unwrap()).map(|pat| Self {
            pattern: pat,
            owners: segments.iter().skip(1).map(|&s| s.to_string()).collect(),
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
