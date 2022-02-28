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

use std::io;
use std::path;
use std::process;

use clap::{arg, command};

mod owners;

fn main() {
    let matches = command!()
        .arg_required_else_help(true)
        .arg(arg!([FILE]).required(true))
        .get_matches();

    let file_name = matches
        .value_of("FILE")
        .expect("required FILE argument was missing");
    let file = path::PathBuf::from(file_name);
    let file = file.canonicalize().unwrap_or_else(|err| {
        eprintln!("Failed to canonicalize input file '{:?}': {}", file, err);
        process::exit(1);
    });

    let maybe_owners_file = find_codeowner_file_for(&file).unwrap_or_else(|err| {
        eprintln!("Error locating CODEOWNERS file: {}", err);
        process::exit(1)
    });

    let owners_file = match maybe_owners_file {
        Some(of) => of,
        None => process::exit(0),
    };

    let maybe_owners = owners::OwnersFile::try_parse(owners_file.clone()).unwrap_or_else(|err| {
        eprintln!(
            "Error parsing CODEOWNERS file at {:?}: {}",
            &owners_file, err
        );
        process::exit(1);
    });

    if let Some(of) = maybe_owners {
        if let Some(all_owners) = of.owner_for(&file) {
            for owner in all_owners {
                println!("{}", owner);
            }
        }
    }

    println!("Hello, world! {:?}", find_codeowner_file_for(&file));
    process::exit(0)
}

// Finds all CODEOWNERS files applicable to the |file|, ordered from
// lowest to highest precedence (i.e. those closest to the repo root
// come first).
fn find_codeowner_file_for(file: &path::Path) -> io::Result<Option<path::PathBuf>> {
    let canonical_file = file.canonicalize()?;
    if !canonical_file.exists() {
        return Err(io::ErrorKind::NotFound.into());
    }

    let repo_root = find_repo_root_for(&canonical_file)?;
    let maybe_codeowners_file = locate_codeowners_in_dir(&repo_root);

    Ok(maybe_codeowners_file)
}

fn locate_codeowners_in_dir<P: AsRef<path::Path>>(dir: P) -> Option<path::PathBuf> {
    let dir = dir.as_ref();

    let github = dir.join(".github").join("CODEOWNERS");
    let docs = dir.join("docs").join("CODEOWNERS");
    let here = dir.join("CODEOWNERS");

    if github.exists() {
        Some(github)
    } else if docs.exists() {
        Some(docs)
    } else if here.exists() {
        Some(here)
    } else {
        None
    }
}

fn find_repo_root_for<P: AsRef<path::Path>>(file: P) -> io::Result<path::PathBuf> {
    let file = file.as_ref();

    if !file.is_absolute() {
        // wat
        panic!("didn't we already canonicalize this?  {:?}", file);
    }

    let canonical_dir = match file.parent() {
        Some(dir) => dir,
        None => return Err(io::ErrorKind::NotFound.into()),
    };

    let output = process::Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .current_dir(canonical_dir)
        .stdout(process::Stdio::piped())
        .output()?;

    if output.status.success() {
        let dirname = String::from_utf8(output.stdout).expect("expected a valid string");
        Ok(path::PathBuf::from(dirname.trim()))
    } else if output.status.code() == Some(128) {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "file is not in a git repository",
        ))
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "wat"))
    }
}
