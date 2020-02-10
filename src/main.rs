use clap::{App, Arg, ArgMatches};
use codeowners::Owner;
use codeowners::Owners;
use git2::Repository;
use std::env::current_dir;
use std::io::{self, BufRead};
use std::path::Path;
use std::path::PathBuf;

type ExitError = (Option<String>, i32);

fn main() {
    if let Err((msg, code)) = run() {
        if let Some(msg) = msg {
            eprintln!("{}", msg);
        }
        std::process::exit(code)
    }
}

fn run() -> Result<(), ExitError> {
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
            if !p.exists() {
                return Err((Some(format!("specified file does not exist: {:?}", p)), 1));
            }
            p.to_path_buf()
        }
        None => discover_codeowners().ok_or_else(|| (Some(format!(concat!(
        "No CODEOWNERS file found in this repo.\n",
            "Ensure one exists at any of the locations documented here:\n",
            "https://help.github.com/en/github/creating-cloning-and-archiving-repositories/about-code-owners#codeowners-file-location\n",
        ))), 1))?,
    };

    let owners = codeowners::from_path(ownersfile);

    match matches.value_of("path").unwrap().as_ref() {
        "-" => {
            let stdin = io::stdin();
            for path in stdin.lock().lines().filter_map(Result::ok) {
                if !resolve(&owners, &matches, &path) {
                    return Err((None, 2));
                }
            }
        }
        path => {
            if !resolve(&owners, &matches, path) {
                return Err((None, 2));
            }
        }
    }

    Ok(())
}

fn resolve(owners: &Owners, matches: &ArgMatches, path: &str) -> bool {
    let (teams, users, emails) = (
        matches.occurrences_of("teams") > 0,
        matches.occurrences_of("users") > 0,
        matches.occurrences_of("emails") > 0,
    );
    let owners = match owners.of(path) {
        Some(owners) => owners,
        None => return false,
    };
    let owned = owners
        .iter()
        .filter_map(|owner| {
            if teams {
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
            }
        })
        .collect::<Vec<_>>();

    if owned.is_empty() {
        return false;
    } else {
        println!("{}", owned.join(" "));
    }

    true
}

fn discover_codeowners() -> Option<PathBuf> {
    let curr_dir = current_dir().unwrap();
    let repo = match Repository::discover(&curr_dir) {
        Ok(r) => r,
        Err(e) => {
            println!(
                "Repo discovery failed. Is {} within a git repository? Error: {}",
                curr_dir.display(),
                e
            );
            return None;
        }
    };

    let repo_root = repo.workdir().unwrap();
    codeowners::locate(&repo_root)
}
