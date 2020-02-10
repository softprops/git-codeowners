# git-codeowners [![Build Status](https://travis-ci.org/softprops/git-codeowners.svg?branch=master)](https://travis-ci.org/softprops/git-codeowners) [![Software License](https://img.shields.io/badge/license-MIT-brightgreen.svg)](LICENSE) [![Crates.io](https://img.shields.io/crates/v/git-codeowners.svg)](https://crates.io/crates/git-codeowners)

> a git extention for Github [CODEOWNERS files](https://help.github.com/articles/about-codeowners/)

Github `CODEOWNERS` files document ownership over paths within git repositories allowing
you to more effectively focus communication with the right people.

## install

### Homebrew

For osx users, you can use `brew` to install or update `git-codeowners`

```bash
$ brew install softprops/tools/git-codeowners
```

To upgrade, just use `brew upgrade` instead

### GH releases

You can download releases for osx and linux directly from github releases

```bash
$ cd $HOME/bin
$ curl -L "https://github.com/softprops/git-codeowners/releases/download/v0.1.2/git-codeowners-$(uname -s)-$(uname -m).tar.gz" \
  | tar -xz
```

### Cargo

If you are a rust user can can just use `cargo`

```bash
$ cargo install git-codeowners
```

## usage

git-codeowners is intended for use as a git extention ( a program whose name starts with git- ) to extend your git workflow.

```bash
$ git codeowners file src/main.rs
@softprops
```

```bash
$ git codeowners log origin/master..
05a4fc chore: do the hokey cokey
 * "Cargo.lock" @ops
 * "Cargo.toml" @ops

e5fe46 feat: Discover current repository
 * "Cargo.lock" @ops
 * "Cargo.toml" @ops
 * "src/main.rs" @softprops

Summary:
 * @softprops: 1 file in 1 commit, including: "src/main.rs" in e5fe46 feat: Discover current repository
 * @ops: 4 files in 2 commits, including: "src/main.rs" in 05a4fc chore: do the hokey cokey
```

```bash
$ git-codeowners --help
git-codeowners 0.1.2
Github CODEOWNERS answer sheet

USAGE:
    git-codeowners [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -e, --emails     Only return emails
    -h, --help       Prints help information
    -t, --teams      Only return teams
    -u, --users      Only return users
    -V, --version    Prints version information

OPTIONS:
    -c, --codeowners <codeowners>    An explicit path for a CODEOWNERS file. program will exit 1 if file can not be
                                     resolved

SUBCOMMANDS:
    help    Prints this message or the help of the given subcommand(s)
    log     annotate log information
    path    Finds information about a specific path or set of paths
```

Doug Tangren (softprops) 2017
