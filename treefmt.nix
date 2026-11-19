{ pkgs, ... }:
{
  # A tracked file no formatter claims and no exclude names fails
  # root:format: a new file type is decided in the change that adds it.
  settings.on-unmatched = "fatal";

  # What no formatter owns, each with its reason.
  settings.excludes = [
    # Markdown: the README, the site's pages and their notes are kept by
    # their writers, and no task reads one for its words or its form
    # (boomdrivedev/blueprints principle 16).
    "*.md"
    "*.mdx"
    # Written by devenv and bun in their own layouts.
    "*.lock"
    # Drawn marks; no formatter keeps an SVG stable.
    "*.svg"
    # Lists in gitignore syntax.
    ".gitignore"
    # The license's text, as its authors wrote it.
    "LICENSE"
    # The site's components: Biome formats only an .astro file's
    # frontmatter, and only experimentally; astro check reads them
    # (site:typecheck).
    "site/**/*.astro"
    # A lookup's recorded answers, kept byte for byte as it gave them.
    "crates/headwater-lookups/tests/answers/**"
    # Build outputs and caches.
    "target/**"
    ".devenv/**"
    ".moon/cache/**"
    "site/node_modules/**"
    "site/dist/**"
    "site/.astro/**"
  ];

  # Rust, by the pinned toolchain's rustfmt (devenv.nix).
  programs.rustfmt.enable = true;
  programs.rustfmt.edition = "2021";

  programs.deadnix.enable = true;
  programs.nixfmt.enable = true;
  settings.formatter.deadnix.pipeline = "nix";
  settings.formatter.deadnix.priority = 1;
  settings.formatter.nixfmt.pipeline = "nix";
  settings.formatter.nixfmt.priority = 2;

  # The release's and the checks' scripts.
  programs.shellcheck.enable = true;
  programs.shfmt.enable = true;
  programs.shfmt.indent_size = 2;
  settings.formatter.shellcheck.pipeline = "shell";
  settings.formatter.shellcheck.priority = 1;
  settings.formatter.shfmt.pipeline = "shell";
  settings.formatter.shfmt.priority = 2;

  # Biome lays out the site's TypeScript, JavaScript, CSS and JSON by the
  # settings biome.json holds, the ones site:format checks and site:lint
  # lints by, so the two never differ. Biome is the package devenv.nix puts
  # in the shell.
  settings.formatter.biome = {
    command = "${pkgs.biome}/bin/biome";
    options = [
      "format"
      "--write"
      "--no-errors-on-unmatched"
    ];
    # treefmt's * crosses directories: each names every such file under
    # site/, its dependencies and outputs excluded above.
    includes = [
      "biome.json"
      "site/*.ts"
      "site/*.mjs"
      "site/*.css"
      "site/*.json"
      # The Boomdrive plugin's configuration schema.
      "plugins/*.json"
    ];
  };

  # .prototools is TOML under a name with no extension, and .versionrc
  # YAML under one.
  programs.taplo.enable = true;
  settings.formatter.taplo.includes = [
    "*.toml"
    ".prototools"
  ];
  programs.yamlfmt.enable = true;
  programs.yamlfmt.settings.formatter = {
    type = "basic";
    retain_line_breaks_single = true;
  };
  settings.formatter.yamlfmt.includes = [
    "*.yml"
    "*.yaml"
    ".versionrc"
  ];
}
