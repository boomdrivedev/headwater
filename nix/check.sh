#!/bin/sh
# The nix project's type check: flake.nix parses as Nix, and flake.lock
# locks every input it declares, each by GitHub. It evaluates nothing
# against a store and needs no network: the build itself is CI's.
set -eu

nix-instantiate --parse flake.nix >/dev/null
for input in nixpkgs rust-overlay; do
  jq -e --arg input "$input" '.nodes.root.inputs[$input]' flake.lock >/dev/null || {
    echo "nix: flake.lock locks no input $input" >&2
    exit 1
  }
done
echo "nix: flake.nix parses, and flake.lock locks its inputs"
