# repository-reading Specification

## Purpose
One reading of a repository state, the checkout as it stands, from which
every answer of a scan is derived, made without changing anything in the
repository (5, 31).

## Requirements

### Requirement: A reading names its repository state

A reading SHALL name the checkout's path, the commit git names for it, and
whether the checkout holds changes its commit does not, read from tracked
files as they stand, committed or not (31).

#### Scenario: A checkout with an uncommitted change

- **WHEN** a checkout whose `flake.lock` was changed after its commit is read
- **THEN** the reading names the commit, marks the state as holding changes, and reads the pins the changed lock gives

### Requirement: Reading changes nothing

Reading a repository state SHALL change nothing in it: files SHALL be read
as plain files, and git SHALL be asked only with `--no-optional-locks`, so
it refreshes no index (5).

#### Scenario: The checkout's bytes before and after

- **WHEN** a checkout is read
- **THEN** every file of the checkout, and git's index, refs and configuration, hold the same bytes as before

### Requirement: The bound dependency descriptions

A reading SHALL be read by every dependency description the binding names,
or, where `.config/headwater.toml` names none, by every description that
recognises one of the repository's declarations, and SHALL name the
descriptions it was read by (51, 67). The Nix flake is the one description
this version holds (C.2).

#### Scenario: A flake with no binding

- **WHEN** a checkout holding `flake.lock` and no `.config/headwater.toml` is read
- **THEN** the reading names `nix-flake` as its one description

#### Scenario: A checkout with no flake

- **WHEN** a checkout holding no `flake.lock` is read
- **THEN** the reading names no description and holds no dependency

### Requirement: A tiers declaration that does not read is unread

Where `.config/headwater.toml` is refused, the reading SHALL name it as an
unread declaration at that path with the reason, and SHALL read the
repository as if it held no such file: the default tier alone, every
recognising description bound (1, C.7).

#### Scenario: A file that does not parse

- **WHEN** a checkout's `.config/headwater.toml` does not parse
- **THEN** the reading holds one unread declaration located at `.config/headwater.toml`, and its configuration is the default
