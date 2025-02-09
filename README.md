# `reposcan` – a manager for git repositories

Command line tool for discovering local git repositories and keeping track of local and remote changes collectively.

## Motivation

Perhaps you find yourself using many git repositories to collaborate and keep track of versions in software development, scientific projects, or even your creative works.
Then a tool such as this might be helpful by
- **discover**ing repositories on your device by scanning directory trees,
- curating a list of repositories for fast interactions,
- checking the collective **status** at a glance, i.e. seeing which repositories contain uncommitted changes,
- **fetch**ing commits to all repositories from all remotes with a single command,
- **list**ing all known repositories.

## Installation

There are currently no precompiled binaries available.
`reposcan` can be compiled from source via rust's `cargo` package manager from [`crates.io`](https://crates.io/crates/reposcan).
Instructions there can be found under the "Install".
If rust is not yet installed, visit [rustup.rs](https://rustup.rs/) for instructions.

## Usage

A quick and crisp entry into using `reposcan` can be found by typing
```bash
reposcan --help
```

### Discovery and curation

As a first step, 
```bash
reposcan discover
```
will search the current directory and subdirectory for git repositories with a file tree.
You will see
1. a list of newly discovered repositories as well as
2. a list of repositories which were curated but not found anymore.

In a second step, you may either add the newly found repositories by typing
```bash
reposcan discover --add
```
or remove obsolete repositories by typing
```bash
reposcan discover --prune
```

Discovery and curation only consider the current working directory and its subdirectories, because these are potentially the most time-consuming operations.
In contrast, checking the status and fetching are performed on all known repositories in the curated list independent of the current working directory.

### Status

Typing
```bash
reposcan status
```
lists changes to repositories that have changed files or are in an in-between state of merging, rebasing, etc.

Clean repositories will not show up.

### Fetching

By executing
```bash
reposcan fetch
```
all known repositories are entered and updated from all remotes.

Currently authentication is not yet implemented, but the plan is to seamlessly obtain authentication in the future.
If one has set up `git` to properly authenticate to a server, then authentication will also work with `reposcan` without the need to set it up seperately.

### Listing

```bash
reposcan list
```

will output all known repositories.

----

This project is not affiliated with the Git™ project nor with the Software Freedom Conservancy.