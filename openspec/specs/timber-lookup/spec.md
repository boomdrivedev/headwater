# timber-lookup Specification

## Purpose
The timber release lookup and notes lookup (C.3, C.4, C.7): for each
identity the binding names for it, the head of the channel the binding
names, as the host's timber serving process answers, dated by the
promotion that moved the channel there and carrying the notes its index
carries.

## Requirements

### Requirement: The host's serving process answers

The lookup SHALL ask the host's timber serving process for the
repository's status at `/<owner>/<name>/status.json`, at `127.0.0.1:7420`
or the address `HEADWATER_TIMBER_ADDRESS` names, and SHALL send nothing
of the repository but the owner and name asked about (37, 38, timber 132).

#### Scenario: No serving process

- **WHEN** nothing answers at the address
- **THEN** the identity is unreached, naming the address and that `timber status --page` in a checkout of the repository starts it

### Requirement: The channel's head alone

The lookup SHALL list one release, the head of the channel the binding
names, for the component the binding names or, where it names none, the
component with no name or the repository's own name. It SHALL be dated by
the promotion that moved the channel to it, linked to the GitHub release
at the version's tag under the dependency's tag scheme, and carry the
notes timber's status gives for that version. A channel with no head
SHALL list none (C.3, C.4, timber 31, 134).

#### Scenario: The stable channel behind the newest version

- **WHEN** sluice publishes 0.22.0 and 0.22.1 and its `stable` channel's head is 0.22.0
- **THEN** the lookup lists 0.22.0 alone, dated by its promotion, linked to `https://github.com/boomdrivedev/sluice/releases/tag/v0.22.0`, with its notes

### Requirement: A refusal is unreached

The lookup SHALL answer unreached, with timber's words, where the status,
its channels or the channel's chain is refused or unreadable, where the
policy declares no such channel, or where the head names no version.
Notes timber could not render, or does not carry for the version, SHALL be
unread with the reason (43, 55).

#### Scenario: A channel the policy does not declare

- **WHEN** the binding names the channel `beta` and timber's status declares none of that name
- **THEN** the identity is unreached, saying timber's policy declares no channel `beta`

### Requirement: The binding gives each identity one lookup

A scan SHALL ask the timber lookup alone for an identity the binding names
for it, while the binding binds it; every other `pkg:github` identity
SHALL be asked of the GitHub releases lookup, while bound; and an identity
neither answers for SHALL be asked of none (51, C.3).

#### Scenario: A flake's tool bound to timber

- **WHEN** a flake pins sluice at 0.21.2 and worktrunk, and the binding names sluice for timber's `stable` channel, whose head is 0.22.0
- **THEN** the report holds 0.22.0 for sluice at level minor, attested by `timber`, GitHub is never asked about sluice, and worktrunk's releases come from GitHub

#### Scenario: A binding that names GitHub's lookup alone

- **WHEN** `[binding] release_lookups` names `github-releases` alone
- **THEN** timber is never asked, and the report's release lookups are `github-releases` alone
