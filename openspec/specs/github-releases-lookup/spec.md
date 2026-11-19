# github-releases-lookup Specification

## Purpose
The GitHub releases lookup (C.3) and the GitHub release notes lookup
(C.4): every release of each `pkg:github` identity's repository, read
through GitHub's GraphQL with the asker's own `gh` sign-in, with each
release's body as its notes.

## Requirements

### Requirement: Fifty repositories a query, through the asker's sign-in

The lookup SHALL ask GitHub's GraphQL through `gh api graphql`, signed in
as the asker, about at most fifty repositories a query, and each query
SHALL name only the repositories asked about and their page cursors (37,
38, C.3).

#### Scenario: Fifty-one repositories

- **WHEN** the lookup is asked about fifty-one identities that each list no release
- **THEN** it sends two queries, and lists no release for each

#### Scenario: What a query names

- **WHEN** the lookup is asked about `boomdrivedev/sluice` and `acme/gone`
- **THEN** its first query names the two repositories and nothing else of the repository asking

### Requirement: Every release, page by page

The lookup SHALL read a repository's releases a hundred at a time, newest
created first, following each page's cursor until none is left, and
SHALL answer unreached where a repository lists more than thirty pages
(43, C.3).

#### Scenario: A second page

- **WHEN** sluice's first page says another follows at a cursor
- **THEN** a second query asks for sluice alone after that cursor, and the listing holds the releases of both pages

### Requirement: A release as GitHub gives it

Each release SHALL be listed with its tag, the version its dependency's tag
scheme reads from the tag, its date (`publishedAt`, else `createdAt`), its
link, GitHub's prerelease flag, and its body as its notes, whole, or that
upstream wrote nothing where the body is empty. A draft SHALL be no
release, and a tag the scheme does not spell SHALL name no version (11,
43, C.1, C.3, C.4).

#### Scenario: A draft and a foreign tag

- **WHEN** sluice's pages hold a draft and a tag the `v` scheme does not spell
- **THEN** neither is listed, and every other release carries its version, date, link, prerelease flag and body

### Requirement: An unreached repository says why

Where `gh` gives no answer, every identity of that query SHALL be
unreached with the reason. Where GitHub answers but gives no releases for a
repository, that identity SHALL be unreached with GitHub's own message.
An identity that is not `pkg:github` SHALL be unreached, saying so (43).

#### Scenario: A missing repository

- **WHEN** GitHub answers a query with an error for `acme/gone`
- **THEN** `acme/gone` is unreached with GitHub's words, and sluice in the same query is listed

#### Scenario: gh fails

- **WHEN** `gh` gives no answer for a query
- **THEN** every identity of that query is unreached with the reason
