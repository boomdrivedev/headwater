# Tasks

This change records headwater v0.1.0 as it landed on main at 2bee268
(#5). Every task below was done by the commit it names, before the
repository kept OpenSpec; each is ticked against that commit, and its
check is the moon task named, which the gate ran at the landing. Every
command runs inside `devenv shell`.

## 1. The toolchain

- [x] 1.1 Rust at the version `.prototools` pins, installed by rustup in the devenv shell, with rustfmt, shellcheck and shfmt in treefmt (ab6ce5b). Verify with `moon run root:toolchain root:format`.

## 2. The core

- [x] 2.1 The reading of a checkout, which writes nothing, by the bound descriptions and the tiers declaration (repository-reading spec; 08ad99a). Verify with `moon run core:test`: `five_and_thirty_one_a_reading_names_its_commit_and_writes_nothing`, `c7_a_tiers_declaration_that_does_not_read_is_named_unread`.
- [x] 2.2 The Nix flake description and the merging of declarations into dependencies (nix-flake-description spec; 08ad99a). Verify with `moon run core:test`: the tests of `flake.rs`, `declaration.rs` and `version.rs`.
- [x] 2.3 The tiers declaration and its binding (tiers-declaration spec; 08ad99a). Verify with `moon run core:test`: the tests of `config.rs`.
- [x] 2.4 The derivation of the report (scan-report spec; 08ad99a). Verify with `moon run core:test`: the nine tests of `tests/report.rs` and `twelve_a_portion_is_cut_at_a_line_end`.

## 3. The lookups

- [x] 3.1 GitHub's releases and release notes through GraphQL (github-releases-lookup spec; 08ad99a). Verify with `moon run lookups:test`: `c3_github_lists_every_release_through_its_pages`, `forty_three_github_unreached`, `c3_fifty_repositories_a_query`.
- [x] 3.2 timber's channel head from the host's serving process (timber-lookup spec; 08ad99a). Verify with `moon run lookups:test`: `s17_timber_lists_the_channels_head_alone`, `forty_three_timber_unreached`.

## 4. The command line

- [x] 4.1 `headwater scan`, its binding of a lookup to each identity, its JSON and text forms, and `--version` (command-line and timber-lookup specs; 08ad99a). Verify with `moon run cli:test`: `s17_the_binding_gives_each_identity_its_lookup`, `fifty_one_the_binding_names_its_lookups`, `the_binary_names_its_version_and_revision`.

## 5. The Boomdrive plugin

- [x] 5.1 The adapter's declaration, `headwater capture`, and one capture per newer release held to weir's conformance pack (release-captures spec; 08ad99a). Verify with `moon run plugin-boomdrive:test cli:test`: `c12_one_capture_per_newer_release_passes_the_pack`, `s6_the_same_release_keeps_its_key`, `c1_the_declaration_reads`, `c1_the_binarys_capture_passes_the_pack`.

## 6. The flake and the projects

- [x] 6.1 The flake exports headwater's package, versioned by the tree, and moon runs one project per crate, for the plugin and for the flake (7c7c02d). Verify with `moon run nix:lint nix:typecheck`.

## 7. The release

- [x] 7.1 Each push to main in which git-cliff finds a new version is released from its release commit, and root holds the toolchain, the versions and the release to it (release-version spec; c0f0c55). Verify with `moon run root:versions root:release`.

## 8. The pages

- [x] 8.1 The README and CLAUDE.md say what the first version does and how its checks and releases run (382b8e4). Read by hand: no task reads a Markdown file (record 0001).

## 9. The record

- [x] 9.1 OpenSpec in the repository, with the skills and commands the family's pinned OpenSpec writes under `.claude/`, and this change validated with `openspec validate --strict` and archived, so `openspec/specs/` holds the nine capabilities. Verify with `openspec validate --specs --strict --no-interactive`, by hand: no moon task runs it yet.
