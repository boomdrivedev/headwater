# Spec Delta

## Purpose

The report a scan derives from one reading and what the lookups answered:
each dependency in its tier, every release newer than its pin with its
level, notes and facts, and every dependency or declaration that could
not be read, with why. The derivation reaches no lookup and writes
nothing (32, 33, 35).

## ADDED Requirements

### Requirement: Newer by the version order alone

A report SHALL hold, for a dependency whose pin is known, every listed
release newer than the pin by Semantic Versioning's precedence, newest
first, and none that is not newer; never ordering by date, spelling or a
lookup's listing order. A release whose order against the pin cannot be
decided SHALL be given apart as undecided, never left out (6, 7, 8, 45).

#### Scenario: Calendar versions

- **WHEN** a dependency pinned at a calendar version lists `2026.10`
- **THEN** no release is newer, and `2026.10` is given as undecided at level undecided

### Requirement: A newer prerelease is held only where asked for

A newer release that is a prerelease, by the lookup's own mark or else by
the version's grammar, SHALL be held only where the pin is a prerelease
or the dependency's tier asks for prereleases; the report SHALL count the
newer prereleases it leaves out (8, C.3).

#### Scenario: A release candidate

- **WHEN** a dependency pinned at 1.4.2 lists 2.0.0-rc.1 marked prerelease, in a tier that does not ask for prereleases
- **THEN** 2.0.0-rc.1 is not held, and one prerelease is counted as left out

#### Scenario: A tier that asks for prereleases

- **WHEN** the same dependency is named by a tier with `prereleases = true`
- **THEN** 2.0.0-rc.1 is held, first, at level prerelease

### Requirement: Every release has a level from the two versions alone

Each release SHALL carry a level against the pin: prerelease for a
prerelease; otherwise major, minor or patch, the first of those parts in
which it differs from the pin, or patch where none does; undecided where
its order against the pin is undecided or either version does not parse.
No level SHALL be read from notes, a date or an advisory (70, 73, S16).

#### Scenario: Notes that call a minor breaking

- **WHEN** a dependency pinned at 1.4.2 lists 1.4.3, and 1.5.0 whose notes say "BREAKING"
- **THEN** 1.5.0 is at level minor and 1.4.3 at level patch

### Requirement: An unknown pin is never nothing newer

Where a dependency's pin is unknown, its newer releases SHALL be null,
never an empty list, its pin SHALL say why it is unknown, and the newest
release listed that parses as a version SHALL be given apart (3).

#### Scenario: A branch pin

- **WHEN** nixpkgs is pinned to a branch and its lookup lists `25.5.0` among others
- **THEN** its newer releases are null, its pin names the branch, and its newest listed is 25.5.0

### Requirement: Each dependency says how it was read

Each dependency SHALL be read where every lookup asked listed it; unread,
with each lookup's reason, where a lookup was not reached, its newer
releases then null; and known to no lookup where no lookup was asked, its
newer releases null. Every other dependency SHALL be reported as usual
(16, 55, 58, S3).

#### Scenario: A lookup answering HTTP 502

- **WHEN** one dependency's lookup is unreached with `HTTP 502` and another's lists its releases
- **THEN** the first is unread naming `HTTP 502` with null newer releases, and the second is read with its newer releases

#### Scenario: An identity no lookup knows

- **WHEN** a dependency is asked of no lookup
- **THEN** it is known to no lookup, with null newer releases

### Requirement: Unread declarations and unplaced dependencies

Every declaration that could not be read SHALL be in every report with its
location and reason. A dependency two tiers name alike SHALL be reported
apart from the tiers, unread, naming both tiers, and every other
dependency SHALL be reported in its tier with the tier's cadence (1, 13,
57, S14).

#### Scenario: Two tiers alike

- **WHEN** the tiers `a` and `b` both name one identity outright
- **THEN** that dependency is unplaced and unread naming `a and b`, and every other dependency is in its tier

### Requirement: Notes whole or a leading portion

Each release SHALL carry, for each notes lookup that answered, the notes
whole and unchanged, or the leading portion asked for in bytes, cut back
to the end of its last whole line, with the whole's size in bytes, its
SHA-256 and whether it is whole; that upstream wrote nothing; or that the
notes are unread, with the reason (11, 12, 46, 55, S7).

#### Scenario: Forty kibibytes of notes

- **WHEN** a release's notes run past 40 KiB and no portion is asked for
- **THEN** the report gives them whole and unchanged, marked whole

#### Scenario: A portion of a thousand bytes

- **WHEN** the same report is asked for 1000 bytes of notes
- **THEN** it gives a leading portion of at most 1000 bytes ending at a line's end, marked not whole, with the whole's size

### Requirement: Range marks

Each release SHALL be marked, for each declaration of its dependency that
gives a range, inside or outside that range, and the marks SHALL never
leave a release out (56, S13).

#### Scenario: A range admitting minors

- **WHEN** a dependency declared at 3.2.0 with range `^3.0` and at 3.1.0 with none lists 3.2.0 and 4.0.0
- **THEN** its pin is 3.1.0, both releases are held, 3.2.0 marked inside the range and 4.0.0 outside it

### Requirement: A release's identity and its facts

Each release SHALL carry its identity, its dependency's Package URL with
its version, and its facts: the identity, its labels, its tier, the pin,
the version, the level, its date's day, advisories `unread`, and the
lookups that list it. Facts its upstream states SHALL be given apart, by
lookup (10, 53, 71, 72).

#### Scenario: The facts of one release

- **WHEN** `pkg:github/acme/ui`, labelled `ui` in the default tier and pinned at 1.0.0, lists 1.0.1 dated 2026-10-12
- **THEN** the release is `pkg:github/acme/ui@1.0.1`, with the facts identity, label `["ui"]`, tier `default`, pin `1.0.0`, version `1.0.1`, level `patch`, date `2026-10-12`, advisories `unread` and its lookup

### Requirement: A release several lookups list is one release

Releases several lookups list at one version SHALL be one release carrying
each lookup's account of it, its tag, date, link and withdrawn mark, and
its prerelease mark SHALL be set where any lookup marks it (39).

#### Scenario: One lookup's account

- **WHEN** timber alone lists sluice 0.22.0
- **THEN** the release's accounts name `timber` alone

### Requirement: Advisories are unread

Each dependency SHALL report its advisories as unread, since no advisory
lookup is bound, and the report's binding SHALL name no advisory lookup
(18, 51).

#### Scenario: Any dependency

- **WHEN** any repository is scanned
- **THEN** each dependency's advisories are unread, saying no advisory lookup is bound

### Requirement: One form, derived the same way every time

A report SHALL name headwater's version, the repository state, when it was
asked, the dependency descriptions and lookups it bound, and SHALL be the
same, field for field, for the same reading, answers and options (32,
35).

#### Scenario: Derived twice

- **WHEN** one reading and one set of answers are derived twice with the same options
- **THEN** the two reports are equal
