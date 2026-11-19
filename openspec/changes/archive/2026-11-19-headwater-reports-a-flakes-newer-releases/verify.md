# Verify: headwater-reports-a-flakes-newer-releases

The report on this change, which records headwater v0.1.0 as main holds
it at 2bee268. The change adds no code, so the check is that each
requirement says what the code does, and that the tests citing its
clauses pass.

## The tests

`cargo test --workspace`, run once at 2bee268 inside `devenv shell` on
2026-11-19: 43 tests passed and none failed or was ignored.

| crate | suite | passed |
|---|---|---|
| `headwater-core` | unit tests | 20 |
| `headwater-core` | `tests/reading.rs` | 2 |
| `headwater-core` | `tests/report.rs` | 9 |
| `headwater-lookups` | `tests/lookups.rs` | 5 |
| `headwater` | `tests/scan.rs` | 4 |
| `headwater-boomdrive` | `tests/capture.rs` | 3 |

## Each spec against the code

- `repository-reading`: `reading.rs` (`read_repository`, `git` with
  `--no-optional-locks`); tests `five_and_thirty_one_…`,
  `c7_a_tiers_declaration_that_does_not_read_is_named_unread`.
- `nix-flake-description`: `flake.rs`, `declaration.rs`
  (`Dependency::pin`, `Deps::merge`), `version.rs` (`TagScheme::of_ref`);
  their unit tests.
- `tiers-declaration`: `config.rs` (`Config::parse`, `place`,
  `timber_for`, `binds_release_lookup`); its unit tests.
- `github-releases-lookup`: `github.rs` (`BATCH` 50, `PAGE` 100,
  `MOST_PAGES` 30, `read_page`); the three GitHub tests of
  `lookups.rs`. The thirty-page limit is read from the code; no test
  reaches it.
- `timber-lookup`: `timber.rs` (`HostServer`, `read_head`) and
  `scan.rs` (`lookup_for`); `s17_timber_lists_the_channels_head_alone`,
  `forty_three_timber_unreached`, and the two binding tests of
  `tests/scan.rs`. The address and `HEADWATER_TIMBER_ADDRESS` are read
  from the code.
- `scan-report`: `report.rs` (`derive`, `dependency_report`,
  `Context::release`, `notes_report`, `cut`); the nine tests of
  `tests/report.rs`.
- `command-line`: `main.rs` and `render.rs`;
  `the_binary_names_its_version_and_revision`. The text form's 600-byte
  default and its lines are read from the code; no test renders text.
- `release-captures`: `plugins/boomdrive/src/lib.rs`, `adapter.toml`,
  `config.schema.json` and `main.rs` (`capture_repo`); the three tests of
  `tests/capture.rs` and `c1_the_binarys_capture_passes_the_pack`.
- `release-version`: `set-version.sh`, `release-commit.sh`,
  `.github/workflows/ci.yml`, held by `root:versions` and
  `root:release`, which the gate ran at the landing.

## Tasks

Every task of `tasks.md`, 1.1 to 9.1, is ticked against the commit that
did it; 9.1 is this change's own.

## The record

`openspec validate headwater-reports-a-flakes-newer-releases --strict`
passed before the archive, and `openspec validate --specs --strict`
after it.
