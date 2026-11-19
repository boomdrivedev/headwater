# Spec Delta

## Purpose

The tiers declaration, `.config/headwater.toml` (C.7): each tier's
cadence, the identities it names outright or by a prefix and whether it
asks for prereleases; the default tier's cadence; and the binding, the
release lookups named and the identities the timber lookup answers for
with the channel each reads (41, 67).

## ADDED Requirements

### Requirement: The default tier

A repository with no `.config/headwater.toml` SHALL have the default tier
alone, named `default`, with a cadence of `7d`, and SHALL bind every
profile that recognises one of its declarations. A file's `[default]`
table SHALL set the default tier's cadence (51, C.7).

#### Scenario: No file

- **WHEN** a repository holds no `.config/headwater.toml`
- **THEN** every dependency is placed in the tier `default`, whose cadence is `7d`

### Requirement: The file is refused where it misnames

The file SHALL be refused, with why, where it does not parse, holds a key
it does not define, names a tier `default`, or names an identity that is
not a Package URL without a version (C.1, C.7).

#### Scenario: An identity with a version

- **WHEN** a tier names the identity `pkg:github/acme/ui@1.0.0`
- **THEN** the file is refused, saying it is not a Package URL without a version

### Requirement: Outright before a prefix

A tier that names an identity outright SHALL place it ahead of one that
names it by a prefix, and a dependency no tier names SHALL be placed in
the default tier (13, 57).

#### Scenario: Outright and prefix

- **WHEN** the tier `family` names the prefix `pkg:github/boomdrivedev/` and the tier `favorites` names `pkg:github/boomdrivedev/sluice` outright
- **THEN** sluice is placed in `favorites`, flume in `family`, and nixpkgs in `default`

### Requirement: Two tiers alike place nothing

Where two tiers name one identity alike, both outright or both by a
prefix, the declaration SHALL place it in neither and SHALL give both
tiers' names (57).

#### Scenario: Two tiers outright

- **WHEN** the tiers `twin` and `other-twin` each name `pkg:github/max-sixty/worktrunk` outright
- **THEN** worktrunk is placed in no tier, and both names are given

### Requirement: The binding names the release lookups

`[binding] release_lookups` SHALL name the release lookups bound; where it
is absent every release lookup is bound. `[[timber]]` entries SHALL name,
outright or by a prefix, the identities the timber lookup answers for,
each with its `channel` and, where the publishing repository publishes
more than one component, its `component`; an outright entry SHALL win
over a prefix (51, 57, 67, C.3, C.7).

#### Scenario: A timber entry by prefix

- **WHEN** a `[[timber]]` entry names the prefix `pkg:github/boomdrivedev/` and the channel `stable`
- **THEN** the timber binding for `pkg:github/boomdrivedev/sluice` reads the channel `stable`
