# command-line Specification

## Purpose
headwater's command line: `headwater scan`, which reports a checkout's
newer releases as text for a person or as the one JSON form a program
reads, from one core (33, 35).

## Requirements

### Requirement: A scan of a checkout

`headwater scan` SHALL read the checkout `--repo` names, the current
directory by default, ask each dependency of its bound lookup, and report
every tier on each run, since this version keeps no held state (31).

#### Scenario: A scan of the current directory

- **WHEN** `headwater scan` runs in a checkout with a flake
- **THEN** it reports the flake's dependencies in their tiers

### Requirement: One JSON document for a program

`headwater scan --json` SHALL print the report as one JSON document, the
form every surface gives, with each release's notes whole unless
`--notes-bytes` asks for a portion (12, 35).

#### Scenario: JSON with whole notes

- **WHEN** `headwater scan --json` runs with no `--notes-bytes`
- **THEN** it prints one JSON document whose notes are whole

### Requirement: Text for a person

Without `--json`, the scan SHALL print a projection of the same report,
holding nothing the JSON form does not: the repository and commit, each
unread declaration, and each tier with its cadence and each dependency's
labels, identity, pin or why it is unknown, and its state, with each newer
release's version, level, date, link and notes. Notes SHALL be given in
600 bytes by default, cut at a line's end with their whole size, and whole
for `--notes-bytes 0` (12, 35).

#### Scenario: Long notes for a person

- **WHEN** a release's notes run past 600 bytes and `headwater scan` runs with no flags
- **THEN** the text gives their leading lines within 600 bytes and says how many bytes they hold and that `--notes-bytes 0` gives them whole

### Requirement: The binary names its version and revision

`headwater --version` SHALL print `headwater <version>, revision
<commit>, built by <rustc>`, the version the tree gives and the commit the
binary was built from.

#### Scenario: A build from main

- **WHEN** a binary built from main's tree is asked its version
- **THEN** it prints `headwater 0.0.0, revision ` followed by the commit and `, built by rustc`
