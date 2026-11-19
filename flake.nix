# headwater as a nix package, for `nix profile install github:boomdrivedev/headwater`
# and for a host's or Boomdrive's flake to pin by its tag, as timber's and
# sluice's are. The development shell is devenv's (devenv.nix). Rust is the
# version .prototools names, so the release is built by the compiler the
# checks ran with. The package's version is the tree's, Cargo.toml's
# [workspace.package], and nothing else: 0.0.0 on main, and the release's at
# a release's tag, which names the release commit that wrote it
# (set-version.sh, release-commit.sh). The binary says that version beside
# the revision it was built from, which this gives it, and the build that
# made it.
{
  description = "headwater: every release newer than what a repository pins, with upstream's notes";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      # nixpkgs refuses x86_64-darwin since 26.11, and no release builds it.
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAll =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            }
          )
        );
      rustVersion = (builtins.fromTOML (builtins.readFile ./.prototools)).rust;
      workspace = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package;
      revision = self.rev or self.dirtyRev or "unknown";
    in
    {
      packages = forAll (
        pkgs:
        let
          rust = pkgs.rust-bin.stable.${rustVersion}.minimal;
          platform = pkgs.makeRustPlatform {
            cargo = rust;
            rustc = rust;
          };
        in
        rec {
          headwater = platform.buildRustPackage {
            pname = "headwater";
            inherit (workspace) version;
            env.HEADWATER_REVISION = revision;
            src = self;
            # The family's shared crates come from weir by the revision
            # Cargo.lock pins, fetched by that revision.
            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };
            cargoBuildFlags = [
              "-p"
              "headwater"
            ];
            # The tests run in the devenv shell, through moon, by layer.
            doCheck = false;
            meta = {
              description = "Every release newer than what a repository pins, with upstream's notes and how big a step it is";
              homepage = "https://github.com/boomdrivedev/headwater";
              license = pkgs.lib.licenses.mit;
              mainProgram = "headwater";
            };
          };
          default = headwater;
        }
      );
    };
}
