# Spec Delta

## Purpose

How headwater itself is released, so that a build of a release's tag
reports that release's version from headwater's own tree, with nothing
handed to the build, as timber's releases do.

## ADDED Requirements

### Requirement: Main's tree names no release

On main, every file that carries headwater's version SHALL give `0.0.0`,
the version no release has, and `set-version.sh`, run on a copy, SHALL
write a release's version into each and change nothing else.

#### Scenario: The versions check

- **WHEN** `root:versions` runs on main's tree
- **THEN** every file that carries the version gives `0.0.0`, and a version written on a scratch copy changes only those files

### Requirement: A release is cut from its release commit

At each push to main in which git-cliff finds a new version since the
newest `v*` tag, CI SHALL make the release commit that writes the version
on the pushed commit, build the flake's package from it for each
platform, tag it and publish the binaries under the tag. The release
commit SHALL have one id wherever it is made.

#### Scenario: The release commit made twice

- **WHEN** `release-commit.sh` makes the release commit of one version on one commit in two scratch repositories
- **THEN** both commits have the same id, and a build of it names that version
