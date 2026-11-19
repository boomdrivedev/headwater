## Why

headwater's first version, v0.1.0, landed on main at 2bee268 (#5), its
code in 08ad99a, before the repository kept OpenSpec. This change records
what that version does, requirement by requirement, so `openspec/specs/`
holds headwater's capabilities as built and every later change amends
them. It describes the code at 2bee268 and changes none of it.

The statement is boomdrivedev/blueprints `design/headwater/requirements.md`
with its model, `design/headwater/models/reconstruction-1.md` and its
rulings. The first version is steps H1 to H3 of `design/capture/plan.md`:
one dependency description, two release lookups, the derivation, the
command line and the Boomdrive plugin's adapter.

## What Changes

- **A reading of a checkout.** One reading of the repository state, named
  by its commit and whether it holds changes, which writes nothing (5,
  31), read by the bound dependency descriptions and the tiers
  declaration (51, 67).
- **The Nix flake dependency description.** Each root input of
  `flake.lock` on GitHub is a `pkg:github` dependency, its pin the version
  its ref spells as a tag, unknown for a branch, a revision or no ref; an
  input of another type is unread naming its type; its claims are the
  lines of `flake.nix` that give its URL (C.2).
- **The tiers declaration.** `.config/headwater.toml`: tiers with their
  cadence, identities outright or by a prefix, prereleases; the default
  tier at seven days; the binding's release lookups and the identities
  timber answers for, each with its channel (C.7).
- **Two release lookups.** GitHub's releases through GraphQL and the
  asker's `gh` sign-in, fifty repositories a query, with each release's
  body as its notes (C.3, C.4); and timber's channel head from the host's
  serving process, for the identities the binding names for it, and then
  no other lookup (C.3, C.4).
- **The report.** Every release newer than the pin by Semantic
  Versioning's precedence, its level from the two versions alone (70),
  newer prereleases held only where asked for (8), undecided releases
  apart (7), notes whole or a portion cut at a line's end (12), range
  marks (56), facts (71, 72); unknown pins, unread lookups, unread
  declarations and dependencies known to no lookup each reported as such
  (1, 3, 16, 58); advisories unread.
- **The command line.** `headwater scan`, as text for a person or one
  JSON document (35), and `headwater --version`.
- **The Boomdrive plugin's adapter.** `headwater capture` prints one
  capture per newer release in the capture contract, keyed by its Package
  URL and version, with its notes as raw material and excerpt and its
  facts in the offer, and a completion naming what could not be listed
  (section 9; millpond C.1, C.12).
- **The release.** main names 0.0.0, and CI releases each new version
  git-cliff finds from its release commit, as timber's are.

Out of scope, each a later step of `design/capture/plan.md`:

- Every other dependency description of C.2 (npm, Python, Cargo, Go,
  mise, GitHub Actions, container images, the watch list) and their moves
  (66), and the other release and notes lookups of C.3 and C.4.
- The held state and the store (C.6): due tiers and cadence (15, 17),
  acknowledgements (A.5), and the last answer kept beside a failure (64).
  This version scans every tier on each run and reads no record.
- Advisories (A.4) and the OSV lookup (C.5): every advisory is unread.
- Briefs (A.6), the Claude Code plugin and the herdr plugin.
- The conformance suite of section 12 over every profile.
- headwater's rules in `docs/architecture/`, plan step H2.

## Capabilities

### New Capabilities

- `repository-reading`: one reading of a checkout, which changes
  nothing, by the bound descriptions and the tiers declaration.
- `nix-flake-description`: a flake's root inputs as dependencies, with
  identity, pin, tag scheme, label and claims.
- `tiers-declaration`: `.config/headwater.toml`'s tiers, default tier and
  binding.
- `github-releases-lookup`: every release of a GitHub repository with its
  body as notes.
- `timber-lookup`: a timber channel's head, and which lookup the binding
  gives each identity.
- `scan-report`: the derivation of the report from a reading and the
  lookups' answers.
- `command-line`: `headwater scan` and `headwater --version`.
- `release-captures`: the Boomdrive plugin's adapter, one capture per
  newer release.
- `release-version`: how headwater is versioned and released.

### Modified Capabilities

None.

## Impact

Docs only: `openspec/` and the OpenSpec skills and commands under
`.claude/`. The code it describes is `crates/headwater-core`,
`crates/headwater-lookups`, `crates/headwater`, `plugins/boomdrive`, the
flake, `.github/workflows/ci.yml` and the release scripts, as they stand
at 2bee268.
