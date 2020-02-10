use clap::{App, Arg, ArgMatches, SubCommand};
use codeowners::Owner;
use codeowners::Owners;
use git2::{Error, Repository};
use std::collections::HashSet;
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
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
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
        .subcommand(SubCommand::with_name("path")
            .about("Finds information about a specific path or set of paths")
            .arg(
                Arg::with_name("path")
                    .help(
                        "Path of file in git repo. if '-' is provided path will \
                         be read from stdin. program will exit 2 if no owners can \
                         be resolved",
                    )
                    .index(1)
                    .required(true)
            )
        )
        .subcommand(SubCommand::with_name("log")
            .about("annotate log information")
            .arg(
                Arg::with_name("revspec")
                    .help("interesting commits, e.g. as HEAD~10, or develop..")
                    .index(1)
                    .multiple(true)
                    .required(true)
            ))
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

    match matches.subcommand() {
        ("path", Some(matches)) => match matches.value_of("path").unwrap().as_ref() {
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
        },
        ("log", Some(matches)) => {
            let repo = Repository::discover(".").expect("dir");
            for revspec in matches.values_of("revspec").expect("required") {
                print_for_revspec(&repo, revspec).map_err(|e| (Some(format!("{:?}", e)), 1))?;
            }
        }
        (_, _) => unreachable!("invalid subcommand"),
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
    let curr_dir = match current_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("error: couldn't find current directory: {:?}", e);
            return None;
        }
    };

    match Repository::discover(&curr_dir) {
        Ok(repo) => match repo.workdir() {
            Some(path) => return codeowners::locate(path),
            None => eprintln!("warning: bare repo has no files"),
        },
        Err(e) => {
            eprintln!(
                "warning: repo discovery failed. Is {:?} within a git repository? Error: {}",
                curr_dir, e
            );
        }
    };

    codeowners::locate(&curr_dir)
}

fn print_for_revspec(repo: &git2::Repository, revspec: &str) -> Result<(), Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_range(revspec)?;
    while let Some(commit) = revwalk.next() {
        let commit = commit?;
        let commit = repo.find_commit(commit).expect("commit");
        let diff = repo
            .diff_tree_to_tree(
                Some(&commit.parent(0).expect("parent").tree().expect("tree")),
                Some(&commit.tree().expect("tree")),
                None,
            )
            .expect("diff");
        let mut deltas = diff.deltas();
        let mut files = HashSet::with_capacity(deltas.len());
        while let Some(delta) = deltas.next() {
            if let Some(path) = delta.new_file().path() {
                files.insert(path.to_path_buf());
            }
            if let Some(path) = delta.old_file().path() {
                files.insert(path.to_path_buf());
            }
        }
        println!("{:?} touched files: {:?}", commit.id(), files);
    }

    Ok(())
}
