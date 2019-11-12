extern crate codeowners;
extern crate clap;
extern crate git2;

use codeowners::Owner;
use clap::{App, Arg};
use git2::Repository;
use std::env::current_dir;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::io::{self, BufRead};

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Github CODEOWNERS answer sheet")
        .arg(
            Arg::with_name("codeowners")
                .help("An explicit path for a CODEOWNERS file. program will exit 1 if file can not be resolved")
                .takes_value(true)
                .short("c")
                .long("codeowners"),
        )
        .arg(
            Arg::with_name("teams")
                .help("Only return teams")
                .short("t")
                .long("teams")
                .conflicts_with("users")
                .conflicts_with("emails"),
        )
        .arg(
            Arg::with_name("users")
                .help("Only return users")
                .short("u")
                .long("users")
                .conflicts_with("teams")
                .conflicts_with("emails"),
        )
        .arg(
            Arg::with_name("emails")
                .help("Only return emails")
                .short("e")
                .long("emails")
                .conflicts_with("teams")
                .conflicts_with("users"),
        )
        .arg(
            Arg::with_name("path")
                .help(
                    "Path of file in git repo. if '-' is provided path will \
                     be read from stdin. program will exit 2 if no owners can \
                     be resolved",
                )
                .index(1)
                .required(true),
        )
        .get_matches();
    let ownersfile = match matches.value_of("codeowners") {
        Some(path) => {
            let p = Path::new(path);
            if p.exists() {
                Some(p.to_path_buf())
            } else {
                None
            }
        }
        None => match discover_codeowners() {
            Some(path) => Some(path),
            None => {
                println!("No CODEOWNERS file found in this repo.");
                println!("Ensure one exists at any of the locations documented here:");
                println!("https://help.github.com/en/github/creating-cloning-and-archiving-repositories/about-code-owners#codeowners-file-location");
                exit(1)
            }
        }
    };

    match ownersfile {
        Some(file) => {
            let resolve = |path: &str| {
                let owners = codeowners::from_path(file.clone());
                let (teams, users, emails) = (
                    matches.occurrences_of("teams") > 0,
                    matches.occurrences_of("users") > 0,
                    matches.occurrences_of("emails") > 0,
                );
                match owners.of(path) {
                    None => exit(2),
                    Some(owners) => {
                        let owned = owners
                            .iter()
                            .filter_map(|owner| if teams {
                                match owner {
                                    &Owner::Team(ref inner) => Some(inner.clone()),
                                    _ => None,
                                }
                            } else if users {
                                match owner {
                                    &Owner::Username(ref inner) => Some(inner.clone()),
                                    _ => None,
                                }
                            } else if emails {
                                match owner {
                                    &Owner::Email(ref inner) => Some(inner.clone()),
                                    _ => None,
                                }
                            } else {
                                Some(owner.to_string())
                            })
                            .collect::<Vec<_>>();
                        if owned.is_empty() {
                            exit(2)
                        } else {
                            println!("{}", owned.join(" "));
                        }
                    }
                }
            };
            match matches.value_of("path").unwrap().as_ref() {
                "-" => {
                    let stdin = io::stdin();
                    for path in stdin.lock().lines().filter_map(Result::ok) {
                        resolve(&path);
                    }
                }
                path => resolve(path),
            }
        }
        _ => exit(1),
    }

}

fn discover_codeowners() -> Option<PathBuf> {
    let curr_dir = current_dir().unwrap();
    let repo = match Repository::discover(&curr_dir) {
        Ok(r) => r,
        Err(e) => {
            println!("Repo discovery failed. Is {} within a git repository? Error: {}", curr_dir.display(), e);
            exit(1)
        },
    };

    let repo_root = repo.workdir().unwrap();
    codeowners::locate(&repo_root)
}
