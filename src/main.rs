extern crate codeowners;
extern crate clap;

use codeowners::Owner;
use clap::{App, Arg};
use std::path::Path;
use std::process::exit;
use std::io::{self, BufRead};

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Github CODEOWNERS answer sheet")
        .arg(
            Arg::with_name("codeowners")
                .help("sets an explicit path for a CODEOWNERS file. program will exit 1 if file can not be resolved")
                .takes_value(true)
                .short("c")
                .long("codeowners"),
        )
        .arg(
            Arg::with_name("teams")
                .help("only return teams")
                .short("t")
                .long("teams")
                .conflicts_with("users")
                .conflicts_with("emails"),
        )
        .arg(
            Arg::with_name("users")
                .help("only return users")
                .short("u")
                .long("users")
                .conflicts_with("teams")
                .conflicts_with("emails"),
        )
        .arg(
            Arg::with_name("emails")
                .help("only return emails")
                .short("e")
                .long("emails")
                .conflicts_with("teams")
                .conflicts_with("users"),
        )
        .arg(
            Arg::with_name("path")
                .help(
                    "path of file in git repo. if '-' is provided path will \
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
        _ => codeowners::locate("."),
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
