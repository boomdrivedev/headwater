# release-captures Specification

## Purpose
headwater's Boomdrive plugin: the adapter millpond runs, which prints, in
the family's capture contract, one capture for each release newer than
its dependency's pin, so millpond lands each once by its key (statement
section 9; millpond C.1, C.12).

## Requirements

### Requirement: The adapter's declaration

The plugin SHALL declare its adapter in `plugins/boomdrive/adapter.toml`
as the command `headwater capture`, the schema of its configuration, an
object naming at most the `repository` to read, and no secrets, since the
lookups use the asker's own sign-ins (38; millpond C.1, C.12).

#### Scenario: The declaration read

- **WHEN** millpond reads the plugin's adapter declaration
- **THEN** it names the command `["headwater", "capture"]`, the schema `config.schema.json` and no secrets

### Requirement: A run reads one checkout

`headwater capture --raw-dir <dir>` SHALL scan the checkout `--repo`
names, else the `repository` of the JSON file `CAPTURE_CONFIG` names, else
the current directory, and SHALL accept `--cursor` and need none, since
the pin alone decides what is new (9; millpond C.1).

#### Scenario: The configuration names the checkout

- **WHEN** a run is started with `CAPTURE_CONFIG` naming a file whose `repository` is a checkout
- **THEN** the run reads that checkout

### Requirement: One capture per newer release

A run SHALL offer one capture for each release newer than its
dependency's pin, in every tier and among the unplaced, each keyed
`release/<type>/<namespace>/<name>/<version>` from its Package URL, of
source `release`, its event time the release's date, titled with its label
and version and linked to the release (10; section 9; millpond C.12).

#### Scenario: Two newer releases of sluice

- **WHEN** a flake pins sluice at 0.97.2, and 0.97.5 and 0.98.0 are newer
- **THEN** the run offers `release/github/boomdrivedev/sluice/0.98.0` and `release/github/boomdrivedev/sluice/0.97.5`, each of source `release`

#### Scenario: The same release on every run

- **WHEN** the same report is captured twice
- **THEN** both runs give the same keys (S6)

### Requirement: The raw material and the excerpt

Each capture SHALL write under the raw place, at its key, the notes whole
as `notes.md` where upstream wrote any and the release as the lookups
answered it as `release.json`. A release with notes SHALL be its own
excerpt: the notes, asserted by the upstream repository, of kind `ask`,
saying the release is out and what the repository pins, and arguing with
each declaration's location and claims. A release with no notes SHALL
carry no excerpt (11, 12; section 9).

#### Scenario: A patch release's excerpt

- **WHEN** sluice 0.97.5's notes are `## v0.97.5\r\n\r\n- a fix\r\n` and the repository pins 0.97.2
- **THEN** `notes.md` holds those bytes, and the excerpt is of kind `ask`, asserted by `boomdrivedev/sluice`, saying `sluice 0.97.5 is released; the repository pins 0.97.2.`, arguing with `flake.lock#/nodes/sluice` and `flake.nix:42`

### Requirement: The facts travel in the offer

Each capture SHALL carry the release's facts as the report gives them,
and the facts its upstream states, each under its name prefixed
`upstream-` (71, 72; weir C.9).

#### Scenario: A level in the facts

- **WHEN** sluice 0.97.5 is captured against a pin of 0.97.2
- **THEN** its facts hold the level `patch` and the identity `pkg:github/boomdrivedev/sluice`

### Requirement: The completion names what could not be listed

A run SHALL end with one completion counting every capture and failure,
whose failures name each unread declaration by its location, each unread
dependency by its identity, and each release that could not be captured,
as one with no date, each with why. Its lines SHALL pass weir's
conformance pack's adapter checks (1; millpond C.1).

#### Scenario: A tarball input and an unreached lookup

- **WHEN** a flake's `rex` input is a tarball and flume's lookup is unreached, beside two newer releases of sluice
- **THEN** the completion lists four, failing `flake.lock#/nodes/rex` and `pkg:github/boomdrivedev/flume`, and the pack finds no breach

#### Scenario: A repository no lookup is asked about

- **WHEN** the binary captures a checkout whose one input is a tarball
- **THEN** it prints one line, a completion naming `type tarball`, which passes the pack, and reaches no network
