# headwater

Reports every release newer than what a repository pins, with upstream's
notes and the advisories that name it. The design is the statement of
invariants in boomdrivedev/blueprints, `design/headwater/requirements.md`;
this repository cites its clauses by number and never restates them. The
design book is at https://boomdrivedev.github.io/blueprints/headwater/.

Nothing is built yet. The order of work is steps H1 to H7 of
`design/capture/plan.md` in boomdrivedev/blueprints.

## Rules

headwater's rules are written at plan step H2, into `docs/architecture/`
with `core.md` binding wherever a change is made, and then imported here.
The decisions behind them are `docs/adr/`, one MADR record each,
proposed until the user's yes. Until the rules are written,
boomdrivedev/blueprints `design/principles.md` binds, and the kit
standard as its `code` profile says (`.config/kit.toml`).

## Checks

Run a repository tool through the shell per command,
`devenv shell -- <command>`. moon runs the checks, one task each, tagged
by layer (`.moon/workspace.yml`); `.config/sluice.toml` says which layers
each branch requires, and `main` lands only through sluice:

| layer | what | who requires it |
|---|---|---|
| `check-fast` | the site's format and lint (Biome), type check (astro check) and build; the root's format (treefmt) | `main` and every bolt |

`devenv shell -- moon run ':#check-fast'` runs the fast layer, and
`moon run <project>:<task>` one check: the projects are `site` (`site/`)
and `root`. Once your work is committed, `sluice exec main` runs the
gate's checks under the host's bound. No task reads a Markdown file for
its words (principle 16; record 0001).

## The site

`site/` is the Starlight site at https://boomdrivedev.github.io/headwater,
published by `.github/workflows/pages.yml`. `site/README.md` says where
its pages and diagrams live.

## Commits

Stage by pathspec and commit the paths you wrote: `git add <paths>`, then
`git commit -m "<subject>" -- <paths>`. Never `-a`, never `add .`.
Conventional Commits subjects, scope the part of the tree touched; the
commit-msg hook refuses any other.
