{
  pkgs,
  lib,
  config,
  ...
}:

let
  # proto's own release build: nixpkgs trails it by minor versions or more.
  # Linux takes the static musl build, so no loader needs patching.
  proto =
    let
      version = "0.63.1";
      target =
        {
          aarch64-darwin = "aarch64-apple-darwin";
          x86_64-darwin = "x86_64-apple-darwin";
          aarch64-linux = "aarch64-unknown-linux-musl";
          x86_64-linux = "x86_64-unknown-linux-musl";
        }
        .${pkgs.stdenv.hostPlatform.system}
          or (throw "proto ${version}: no release build for ${pkgs.stdenv.hostPlatform.system}");
      hashes = {
        aarch64-apple-darwin = "8bfde4c862381eeb6e82d0858bb6215732cbd1e384e4eae45da8fa08095e0ffd";
        x86_64-apple-darwin = "aaaff78c7125de1b765f00177f35da75ee16fb1363e07632ace27213ff5d893b";
        aarch64-unknown-linux-musl = "111f5b6a38d3979ebafbc095c8d91ef69c99d1f018addb3582179d4ab91b3441";
        x86_64-unknown-linux-musl = "8596b98ccd833d1203e71931ba25e866d3239f9d3859c922a035a7b830f29d17";
      };
    in
    pkgs.stdenvNoCC.mkDerivation {
      pname = "proto_cli";
      inherit version;
      src = pkgs.fetchurl {
        url = "https://github.com/moonrepo/proto/releases/download/v${version}/proto_cli-${target}.tar.xz";
        sha256 = hashes.${target};
      };
      sourceRoot = "proto_cli-${target}";
      installPhase = ''
        runHook preInstall
        install -Dm755 -t $out/bin proto proto-shim
        runHook postInstall
      '';
    };

  # The application toolchain .prototools pins, on PATH. proto's home is
  # this repository's, in the git common dir so every worktree shares it,
  # and only this repository's pins are read (no global .prototools).
  protoHome = ''
    PROTO_HOME="$(${lib.getExe pkgs.git} rev-parse --path-format=absolute --git-common-dir)/proto"
    export PROTO_HOME
  '';
  activate = ''
    ${protoHome}
    eval "$(${proto}/bin/proto activate bash --export --no-bin --config-mode upwards 2>/dev/null)"
  '';

  # treefmt's rustfmt: the pinned toolchain's.
  rustfmt = pkgs.writeShellApplication {
    name = "rustfmt";
    text = ''
      ${activate}
      exec rustfmt "$@"
    '';
  };
in
{
  # The system tools are this file's, pinned by devenv.lock; moon and Rust
  # are .prototools', installed by the headwater:proto:install task on
  # entering the shell and activated by enterShell.
  packages = [
    pkgs.git
    pkgs.jq
    proto
    # The release's notes and next version, from the conventional commits
    # (cliff.toml; .github/workflows/ci.yml).
    pkgs.git-cliff
    # The nix project's lint and type check (nix/moon.yml).
    pkgs.statix
    pkgs.nix
    # The site (site/): bun builds it, Biome formats and lints its sources
    # (site:format, site:lint, and treefmt for root:format), and Playwright
    # drives the headless Chromium its screenshots are taken in
    # (site/scripts/shots.mjs).
    pkgs.bun
    pkgs.biome
    pkgs.playwright-test
  ];

  env.PLAYWRIGHT_BROWSERS_PATH = "${pkgs.playwright-driver.browsers-chromium}";

  # moon's task archives (.moon/cache/outputs) are a link into the git common
  # dir, so every worktree shares them; the rest of .moon/cache records the
  # checkout's absolute paths and stays its own. moon's home is the
  # repository's too, so no moon of the user's runs in a task.
  enterShell = ''
    if _moon_root="$(git rev-parse --show-toplevel 2>/dev/null)" &&
      _moon_common="$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)"; then
      _moon_cache="$_moon_root/.moon/cache"
      [ -L "$_moon_cache" ] && rm -f "$_moon_cache"
      mkdir -p "$_moon_common/moon-outputs" "$_moon_cache"
      if [ ! -L "$_moon_cache/outputs" ]; then
        rm -rf "$_moon_cache/outputs"
        ln -s "$_moon_common/moon-outputs" "$_moon_cache/outputs"
      fi
      MOON_HOME="$_moon_common/moon-home"
      export MOON_HOME
    fi
    unset _moon_root _moon_common _moon_cache
    ${activate}
  '';

  # The tool versions .prototools lists, installed before enterShell
  # activates them and before treefmt formats with their rustfmt. Rust is
  # rustup's first, with the profile and components rust-toolchain.toml
  # names, as timber's is: proto makes the toolchain's directory before its
  # Rust plugin asks `rustup toolchain list`, and would otherwise find an
  # empty toolchain listed and install nothing. A task, not part of
  # enterShell, so a script that only sources the shell's environment never
  # installs.
  tasks."headwater:proto:install" = {
    exec = ''
      cd "$(${lib.getExe pkgs.git} rev-parse --show-toplevel)"
      ${pkgs.rustup}/bin/rustup toolchain install
      ${protoHome}
      ${proto}/bin/proto install --config-mode upwards
    '';
    before = [
      "devenv:enterShell"
      "devenv:treefmt:run"
    ];
  };

  # Every formatter treefmt.nix configures, which root:format checks.
  treefmt.enable = true;
  treefmt.config = {
    imports = [ ./treefmt.nix ];
    projectRootFile = "devenv.nix";
    programs.rustfmt.package = rustfmt;
  };

  # convco refuses a commit message that is not a conventional commit
  # (.versionrc), installed into the common hooks directory on entering the
  # shell.
  git-hooks.hooks.convco = {
    enable = true;
    settings.configPath = ".versionrc";
  };

  # devenv 2.4 writes the worktree's absolute config path into the hook,
  # which every worktree shares; a plain `prek install` writes none, and prek
  # reads the config at the top of the worktree a commit is in. Remove once
  # devenv installs without the path (cachix/devenv#3270).
  tasks."devenv:git-hooks:install".exec = lib.mkForce ''
    set -euo pipefail
    hooks="$(${lib.getExe pkgs.git} rev-parse --path-format=absolute --git-path hooks)"
    if ! grep -q 'File generated by prek' "$hooks/commit-msg" 2>/dev/null || grep -q -- '--config=' "$hooks/commit-msg"; then
      ${lib.getExe config.git-hooks.package} install --force -t commit-msg
    fi
  '';

  # What CI would release from here: the next version and its notes.
  scripts.release-preview = {
    description = "The next release's version and notes: git cliff --unreleased --bump";
    exec = ''
      set -euo pipefail
      cd "$(git rev-parse --show-toplevel)"
      git cliff --unreleased --bump --strip all "$@"
    '';
  };

  # `devenv shell -- site-dev`: the site at http://localhost:4321/headwater/.
  scripts.site-dev = {
    description = "The site, served with live reload: astro dev";
    exec = ''
      set -euo pipefail
      cd "$(git rev-parse --show-toplevel)/site"
      bun install --frozen-lockfile
      exec bun run dev "$@"
    '';
  };
}
