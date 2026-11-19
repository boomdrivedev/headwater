# Spec Delta

## Purpose

The Nix flake dependency description (C.2): each input of the root of a
repository's `flake.lock` as a dependency, with its identity, its pin, its
label and its claims, read from the lock alone with no Nix evaluated.

## ADDED Requirements

### Requirement: Each root input is a declaration

The description SHALL read each input of the lock's root node as one
declaration, located at `flake.lock#/nodes/<node>`, labelled by the
input's name, and SHALL read no input that follows another's, whose
target is a path rather than a node's name (C.2, 53).

#### Scenario: A followed input

- **WHEN** the lock's root names the inputs `sluice`, `nixpkgs`, `pinned`, `rex` and `followed`, and `followed` names the path `["sluice", "nixpkgs"]`
- **THEN** no declaration is read for `followed`, and `sluice` is declared at `flake.lock#/nodes/sluice` under the label `sluice`

### Requirement: An input on GitHub has a Package URL identity

An input whose original reference the lock records as of type `github`
SHALL have the identity `pkg:github/<owner>/<repo>`, both lowercased. An
input of any other type, or naming no owner and repository, SHALL be
reported unread, naming its type or what it lacks (1, 2, C.1, C.2).

#### Scenario: A tarball input

- **WHEN** the lock's root names an input of type `tarball`
- **THEN** it is an unread declaration whose reason names the type `tarball`, and it is no dependency

### Requirement: The pin is the version the ref spells as a tag

The pin SHALL be the version the input's recorded ref spells as a tag,
in Semantic Versioning's normal spelling, and the text before its first
digit SHALL be the dependency's tag scheme. The pin SHALL be unknown, with
why, where the ref spells no version, as a branch does, or the input names
a revision or no ref (3, C.1, C.2).

#### Scenario: A tag ref

- **WHEN** the input `sluice` records the ref `v0.97.2`
- **THEN** its pin is `0.97.2` and its tag scheme's prefix is `v`

#### Scenario: A prefixed tag ref

- **WHEN** an input records the ref `cli-v1.2.3`
- **THEN** its pin is `1.2.3` and its tag scheme's prefix is `cli-v`

#### Scenario: A branch or a revision

- **WHEN** `nixpkgs` records the ref `nixos-unstable` and another input records only a revision
- **THEN** each pin is unknown, the first naming `nixos-unstable` and the second naming the revision

### Requirement: The claims are the lines of flake.nix that give the URL

Each declaration's claims SHALL be the lines of `flake.nix`, as
`flake.nix:<line>`, that name the input's `github:<owner>/<repo>` URL with
the repository's name ending there, read without regard to case
(vocabulary: claims; C.2).

#### Scenario: A longer name is no claim

- **WHEN** `flake.nix` names `github:boomdrivedev/sluice/v0.97.2` on line 3 and `github:boomdrivedev/sluice-extra` on line 4
- **THEN** the claims of `sluice` are `flake.nix:3` alone

### Requirement: A lock that does not parse is unread

A `flake.lock` that does not parse, or holds no nodes or no root inputs,
SHALL be one unread declaration at `flake.lock` with the reason, never a
set of unknown pins. A repository with no `flake.lock` SHALL not bind the
description (1, 51, C.2).

#### Scenario: A broken lock

- **WHEN** `flake.lock` holds text that is not JSON
- **THEN** the description reads no dependency and one unread declaration located at `flake.lock`

### Requirement: Declarations of one identity are one dependency

Declarations of one identity SHALL be one dependency, wherever they sit.
Its pin SHALL be the oldest of its declarations' pins, and unknown, naming
why, where any declaration's pin is unknown or two pins cannot be ordered
(2, 3, 4).

#### Scenario: Two known pins

- **WHEN** one identity is declared at `3.2.0` and at `3.1.0`
- **THEN** it is one dependency with both declarations, and its pin is `3.1.0`

#### Scenario: Pins that cannot be ordered

- **WHEN** one identity is declared at `2026.10` and at `2026.9`
- **THEN** its pin is unknown, saying the two cannot be ordered
