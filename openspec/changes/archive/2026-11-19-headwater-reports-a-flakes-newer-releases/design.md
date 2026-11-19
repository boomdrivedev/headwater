## Context

headwater v0.1.0 was built before this repository kept OpenSpec, from the
statement's clauses and its model, `design/headwater/models/reconstruction-1.md`
with its rulings. This design records the shape the code took, so later
changes amend it from here.

## Goals / Non-Goals

**Goals:**

- Describe what the code at 2bee268 does, each requirement checked
  against the code and a test that cites its clause.

**Non-Goals:**

- Any change to the code, the checks or the release.
- The statement's clauses this version does not build (the proposal's
  out-of-scope list).

## Decisions

### D1. One derivation over one reading

`report = derive(reading, answers, options)` (model section 1). The
reading (`headwater_core::reading`) is made once, with git asked only
with `--no-optional-locks`; the lookups are asked in `headwater::scan`;
the derivation (`headwater_core::report`) reaches no lookup and writes
nothing, so the same reading, answers and options give the same report
(32). The command line's text and the adapter's captures are projections
of that one report (33, 35).

### D2. The seams are traits

A release lookup is `ReleaseLookup::list_releases`, asked for many
identities at once (model section 6). GitHub's lookup is generic over a
`GraphQl` and timber's over a `StatusSource`, so the tests run both
against recorded answers (`crates/headwater-lookups/tests/answers/`) and
no test reaches the network.

### D3. The order and the level are the core's

Every profile of this version orders and levels by Semantic Versioning
(`headwater_core::version`): precedence where both versions parse, the
same where spelled alike, undecided otherwise. A level is read from the
pin and the version alone (70).

### D4. The binding gives each identity one lookup

`headwater::scan::lookup_for`: timber where `[[timber]]` names the
identity and the binding binds timber, and then no other (C.3); GitHub
for every other `pkg:github` identity while bound; none otherwise, which
the report gives as known to no lookup (58).

### D5. No held state yet

This version keeps nothing between runs. Every scan reaches the lookups
of every tier, the pin alone decides what is new (9), and the adapter
takes `--cursor` and ignores it.

## Risks / Trade-offs

- A scan asks every lookup every run, since nothing marks a tier due. The
  held state of C.6 removes this.
- A repository listing more than 3,000 GitHub releases is unreached,
  never truncated silently.
