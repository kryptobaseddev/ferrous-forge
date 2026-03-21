# @task T023
# @epic T014
{
  description = "Ferrous Forge - System-wide Rust development standards enforcer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        
        ferrous-forge = pkgs.callPackage ./default.nix { };
      in
      {
        packages = {
          default = ferrous-forge;
          ferrous-forge = ferrous-forge;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = ferrous-forge;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            ferrous-forge
            rust-bin.stable.latest.default
            cargo-audit
            cargo-outdated
          ];
        };
      });
}
