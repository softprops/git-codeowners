# git-codeowners [![Build Status](https://travis-ci.org/softprops/git-codeowners.svg?branch=master)](https://travis-ci.org/softprops/git-codeowners)

> a git extention for Github [CODEOWNERS files](https://help.github.com/articles/about-codeowners/)

## usage

git-codeowners is intended for use as a git extention ( a program whose name starts with git- ) to extend your git workflow.

```bash
$ git codeowners src/main.rs
@softprops
```

```bash
$ git-codeowners --help
git-codeowners 0.1.0
Github CODEOWNERS answer sheet

USAGE:
    git-codeowners [FLAGS] [OPTIONS] <path>

FLAGS:
    -e, --emails     Only return emails
    -h, --help       Prints help information
    -t, --teams      Only return teams
    -u, --users      Only return users
    -V, --version    Prints version information

OPTIONS:
    -c, --codeowners <codeowners>    An explicit path for a CODEOWNERS file. program will exit 1 if file can not be resolved

ARGS:
    <path>    Path of file in git repo. if '-' is provided path will be read from stdin. program will exit 2 if no owners can be resolved
```

Doug Tangren (softprops) 2017