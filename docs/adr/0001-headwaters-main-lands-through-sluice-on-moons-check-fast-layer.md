---
number: 1
title: headwater's main lands through sluice on moon's check-fast layer, and no task reads Markdown
status: proposed
date: 2026-11-16
decision-makers:
- the brief of the bolt headwaters-main-is-gated-by-sluice
- the chore headwaters-main-is-gated-by-sluice's code stage
---

# headwater's main lands through sluice on moon's check-fast layer, and no task reads Markdown

Status: proposed, until the user has seen it as a record. The choices
below that the bolt's brief does not name are the code stage's own, made
on the rules written and cheap to change.

Deciders: the brief of the bolt `headwaters-main-is-gated-by-sluice`:
"headwater's main lands only through sluice, its checks tagged by layer
under moon as the family's kits run them, and its CLAUDE.md names
blueprints at boomdrivedev and its rules once written." The chore's code
stage, on the rest.

Sources: boomdrivedev/blueprints `design/principles.md` 16 at `3a5c19d`;
`design/capture/plan.md` step H2; boom's record 0002 and core.5, the
pattern this repository copies; flume's commit `ff61588`, for the theme's
address.

## Context and problem statement

`.config/sluice.toml` already required the tag `check-fast` on `main`
and on every bolt, and moon tagged one task with it, the site's build.
The family's kits each run one task per check, format, lint, type check
and test, each tagged with its layer, and principle 16 forbids a check
that reads a Markdown file. headwater has no code yet: its tree is the
site, its configuration and its Nix shell.

The site's install also failed on a fresh cache: site-theme moved from
agentplot to boomdrivedev, and GitHub's tarball of v0.1.3 now unpacks
into a folder of the new owner's name, so `bun.lock`'s integrity no
longer matched.

## Considered options

For the site's checks:

1. Its build alone, as timber's and flume's sites run.
2. Its format and lint by Biome, its types by `astro check`, and its
   build, one task each, as boom's site runs.

For what spans the tree:

1. Nothing: a file outside the site is checked by no task.
2. `root:format`, treefmt over every tracked file, failing a file no
   formatter claims and no exclude names, as boom's root runs.

For the site's prose:

1. Vale over the pages in `check-fast`, as eddy, millpond and sounding
   seeded it.
2. No prose task.

## Decision outcome

The site's checks are option 2, and the root's option 2. Each is one
moon task tagged `check-fast`, which sluice requires on `main` and on
every bolt. Biome's settings are boom's, so the family's sites lay out
alike; treefmt runs Biome over the same files, so `root:format` and
`site:format` never differ. The site's prose is option 2: principle 16
forbids a task that reads a page, and headwater never had one.

The site takes site-theme from boomdrivedev at the same tag, with the
folder and integrity its tarball now has, as flume's lock holds them.

### Consequences

- The site's sources are laid out by Biome once, in the change that adds
  the check, and two of them change to pass its lint: a `forEach` that
  returned a value is a `for` loop, and a `let` never reassigned is a
  `const`.
- A new file type fails `root:format` until the change that adds it
  names a formatter or an exclude for it.
- `site:install` reaches the registry, so it is never replayed; headwater's
  sluice runs no sandbox, so the gate may reach it.
- headwater's rules are not written yet. When plan step H2 writes
  `docs/architecture/core.md`, its rule for the gate cites this record.
